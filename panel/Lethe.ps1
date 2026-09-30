param(
    [Parameter(Mandatory = $false)]
    [string]$HomePath
)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName Microsoft.VisualBasic

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$binaryCandidates = @(
    (Join-Path $PSScriptRoot 'Lethe.exe'),
    (Join-Path $projectRoot 'Lethe.exe'),
    (Join-Path $projectRoot 'target\release\lethe.exe'),
    (Join-Path $projectRoot 'target\debug\lethe.exe')
)
$letheBinary = $binaryCandidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1

if (-not $letheBinary) {
    [System.Windows.Forms.MessageBox]::Show(
        'No se encontro Lethe.exe. Ejecuta cargo build --release.',
        'Lethe',
        'OK',
        'Error'
    ) | Out-Null
    exit 2
}

if (-not $HomePath) {
    $HomePath = if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'lethe.conf')) {
        $PSScriptRoot
    } else {
        Split-Path -Parent $letheBinary
    }
}

$HomePath = $HomePath.Trim().Trim('"')
$HomePath = [System.IO.Path]::GetFullPath($HomePath)
$letheBinary = [System.IO.Path]::GetFullPath($letheBinary)
$safeWorkingDirectory = [System.IO.Path]::GetTempPath()
[System.Environment]::CurrentDirectory = $safeWorkingDirectory
Set-Location -LiteralPath $safeWorkingDirectory

$script:currentState = 'UNKNOWN'
$script:lastMode = ''
$script:mountPending = $false
$script:pendingSince = [datetime]::MinValue
$script:refreshing = $false
$script:allowFormClose = $false
$script:veracryptBinary = $null
$script:mountLetter = ''

function Test-LethePath {
    param(
        [AllowNull()][object]$Path,
        [ValidateSet('Any', 'Leaf', 'Container')][string]$Kind = 'Any'
    )

    if ($Path -isnot [string] -or [string]::IsNullOrWhiteSpace($Path)) {
        return $false
    }
    if ($Path.IndexOfAny([System.IO.Path]::GetInvalidPathChars()) -ge 0) {
        return $false
    }

    switch ($Kind) {
        'Leaf' { return [System.IO.File]::Exists($Path) }
        'Container' { return [System.IO.Directory]::Exists($Path) }
        default {
            return [System.IO.File]::Exists($Path) -or [System.IO.Directory]::Exists($Path)
        }
    }
}

function Resolve-LetheRuntime {
    $currentHome = $HomePath
    $currentBinary = $letheBinary

    if ($currentHome -and $currentBinary -and
        (Test-LethePath $currentHome 'Container') -and
        (Test-LethePath $currentBinary 'Leaf') -and
        (Test-LethePath ([System.IO.Path]::Combine($currentHome, 'lethe.conf')) 'Leaf')) {
        return $true
    }

    $matches = @()
    foreach ($letterCode in [byte][char]'D'..[byte][char]'Z') {
        $candidateHome = "{0}:\Lethe" -f [char]$letterCode
        $candidateBinary = [System.IO.Path]::Combine($candidateHome, 'Lethe.exe')
        $candidateConfig = [System.IO.Path]::Combine($candidateHome, 'lethe.conf')
        if ((Test-LethePath $candidateBinary 'Leaf') -and
            (Test-LethePath $candidateConfig 'Leaf')) {
            $matches += [pscustomobject]@{
                Home   = $candidateHome
                Binary = $candidateBinary
            }
        }
    }

    if ($matches.Count -ne 1) {
        return $false
    }

    $script:HomePath = $matches[0].Home
    $script:letheBinary = $matches[0].Binary
    $script:veracryptBinary = $null
    return $true
}

function Resolve-VeraCryptBinary {
    if ($script:veracryptBinary -and
        (Test-LethePath $script:veracryptBinary 'Leaf')) {
        return $script:veracryptBinary
    }

    $candidates = @(
        ([System.IO.Path]::Combine($HomePath, 'tools', 'VeraCrypt', 'VeraCrypt.exe')),
        $env:LETHE_VERACRYPT,
        'C:\Program Files\VeraCrypt\VeraCrypt.exe',
        'C:\Program Files (x86)\VeraCrypt\VeraCrypt.exe'
    ) | Where-Object { $_ }

    $script:veracryptBinary = $candidates |
        Where-Object { Test-LethePath $_ 'Leaf' } |
        Select-Object -First 1
    return $script:veracryptBinary
}

function Invoke-Lethe {
    param([Parameter(Mandatory)][string]$Command)

    if (-not (Resolve-LetheRuntime)) {
        return [pscustomobject]@{
            ExitCode = 2
            Output   = ''
            Error    = 'ERROR [RuntimeMissing]: No se encontro una unica memoria con la carpeta Lethe.'
        }
    }

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $letheBinary
    $startInfo.Arguments = $Command
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.WorkingDirectory = $HomePath
    $startInfo.EnvironmentVariables['LETHE_HOME'] = $HomePath
    $resolvedVeraCrypt = Resolve-VeraCryptBinary
    if ($resolvedVeraCrypt) {
        $startInfo.EnvironmentVariables['LETHE_VERACRYPT'] = $resolvedVeraCrypt
    }

    try {
        $process = [System.Diagnostics.Process]::Start($startInfo)
        $output = $process.StandardOutput.ReadToEnd().Trim()
        $errorOutput = $process.StandardError.ReadToEnd().Trim()
        $process.WaitForExit()
        return [pscustomobject]@{
            ExitCode = $process.ExitCode
            Output   = $output
            Error    = $errorOutput
        }
    } catch {
        return [pscustomobject]@{
            ExitCode = 2
            Output   = ''
            Error    = "ERROR [ProcessStartFailed]: $($_.Exception.Message)"
        }
    }
}

function ConvertFrom-LetheStatus {
    param([Parameter(Mandatory)][string]$Text)

    $status = @{}
    foreach ($line in ($Text -split "`r?`n")) {
        $pair = $line -split '=', 2
        if ($pair.Count -eq 2) {
            $status[$pair[0]] = $pair[1]
        }
    }
    return $status
}

function Get-PublicError {
    param(
        [string]$ErrorText,
        [int]$ExitCode = 2
    )

    $code = if ($ErrorText -match 'ERROR \[([^\]]+)\]') { $Matches[1] } else { 'Unknown' }
    $detail = if ($ErrorText -match '^ERROR \[[^\]]+\]:\s*(.+)$') { $Matches[1].Trim() } else { '' }
    switch ($code) {
        'RuntimeMissing' { return 'No se encontro la carpeta Lethe. Reconecta una sola memoria Lethe y pulsa Actualizar.' }
        'ContainerMissing' { return 'No se encontro el contenedor cifrado.' }
        'DependencyMissing' { return 'No se encontro el motor criptografico.' }
        'DependencyInspectionFailed' { return 'No se pudo validar el motor criptografico.' }
        'SignatureInvalid' { return 'La firma del motor criptografico no es valida.' }
        'VersionUnsupported' { return 'La version instalada del motor no es compatible.' }
        'DriveOccupied' { return 'La letra configurada ya esta ocupada.' }
        'VolumeNotMounted' { return 'No hay un volumen abierto para cerrar.' }
        'DriveStillMounted' { return 'El volumen continua abierto.' }
        'ProcessStartFailed' { return "No se pudo iniciar el componente necesario. Codigo: $code." }
        'ProcessFailed' {
            if ($detail) { return "VeraCrypt rechazo la operacion: $detail" }
            return "VeraCrypt rechazo la operacion (salida $ExitCode)."
        }
        'RestoreTargetUnsafe' { return 'El disco virtual fue rechazado por seguridad.' }
        'RestoreIdentityChanged' { return 'El disco virtual cambio despues de crear el plan.' }
        'RestoreConfirmationInvalid' { return 'La confirmacion de restauracion no es valida.' }
        'RestoreFailed' { return 'La restauracion no pudo completarse.' }
        'PhysicalRestoreIdentityChanged' { return 'La identidad de la memoria cambio despues del plan.' }
        'PhysicalRestoreConfirmationInvalid' { return 'La confirmacion fisica no es valida.' }
        'PhysicalRestoreFailed' { return 'La restauracion de la memoria no pudo completarse.' }
        default {
            if ($detail) { return "No se pudo completar la operacion: $detail" }
            return "No se pudo completar la operacion. Codigo: $code; salida: $ExitCode."
        }
    }
}

function Set-StatusMessage {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][System.Drawing.Color]$Color
    )
    $statusLabel.Text = $Text
    $statusLabel.ForeColor = $Color
}

function Invoke-PanelAction {
    param([Parameter(Mandatory)][scriptblock]$Action)

    try {
        & $Action
    } catch {
        $exceptionName = $_.Exception.GetType().Name
        Set-StatusMessage "Error controlado del panel: $exceptionName. Pulsa Actualizar." ([System.Drawing.Color]::Firebrick)
    }
}

function Set-ControlsForState {
    param(
        [Parameter(Mandatory)][hashtable]$Snapshot,
        [switch]$PreserveMessage
    )

    $state = if ($Snapshot.ContainsKey('STATE')) { $Snapshot.STATE } else { 'NOT_READY' }
    $script:currentState = $state
    $letter = if ($Snapshot.ContainsKey('LETTER')) { $Snapshot.LETTER } else { '?' }
    $script:mountLetter = $letter
    $version = if ($Snapshot.ContainsKey('ENGINE_VERSION')) { $Snapshot.ENGINE_VERSION } else { '' }
    $container = if ($Snapshot.ContainsKey('CONTAINER')) { $Snapshot.CONTAINER } else { '' }
    $homeDisplay = $HomePath.TrimEnd('\')
    $diagnosticBox.Text = "Estado real: $state`r`nMemoria Lethe: $homeDisplay`r`nUnidad cifrada: $letter`:`r`nVeraCrypt: $version`r`nContenedor: $container"

    switch ($state) {
        'AVAILABLE' {
            $readOnlyButton.Enabled = $true
            $writeCheck.Enabled = $true
            $writeButton.Enabled = $writeCheck.Checked
            $closeButton.Enabled = $false
            $openFolderButton.Enabled = $false
            $restoreButton.Enabled = $true
            $physicalRestoreButton.Enabled = $true
            if (-not $PreserveMessage) {
                Set-StatusMessage 'Bloqueado. Listo para abrir.' ([System.Drawing.Color]::DarkSlateGray)
            }
        }
        'MOUNTED' {
            $readOnlyButton.Enabled = $false
            $writeCheck.Enabled = $false
            $writeButton.Enabled = $false
            $closeButton.Enabled = $true
            $openFolderButton.Enabled = $true
            $restoreButton.Enabled = $false
            $physicalRestoreButton.Enabled = $false
            $script:mountPending = $false
            if ($script:lastMode -eq 'rw') {
                Set-StatusMessage "Volumen abierto con escritura en $letter`:" ([System.Drawing.Color]::DarkGreen)
            } elseif ($script:lastMode -eq 'ro') {
                Set-StatusMessage "Volumen abierto en solo lectura en $letter`:" ([System.Drawing.Color]::DarkGreen)
            } else {
                Set-StatusMessage "Volumen VeraCrypt abierto en $letter`:" ([System.Drawing.Color]::DarkGreen)
            }
        }
        'OCCUPIED' {
            $readOnlyButton.Enabled = $false
            $writeCheck.Enabled = $false
            $writeButton.Enabled = $false
            $closeButton.Enabled = $false
            $openFolderButton.Enabled = $false
            $restoreButton.Enabled = $false
            $physicalRestoreButton.Enabled = $false
            Set-StatusMessage "$letter`: esta ocupada por otro dispositivo." ([System.Drawing.Color]::Firebrick)
        }
        default {
            $readOnlyButton.Enabled = $false
            $writeCheck.Enabled = $false
            $writeButton.Enabled = $false
            $closeButton.Enabled = $false
            $openFolderButton.Enabled = $false
            $restoreButton.Enabled = $true
            $physicalRestoreButton.Enabled = $true
            if (-not $PreserveMessage) {
                Set-StatusMessage 'Lethe no esta preparado.' ([System.Drawing.Color]::Firebrick)
            }
        }
    }
}

function Refresh-LetheStatus {
    param([switch]$PreserveMessage)

    if ($script:refreshing) { return }
    $script:refreshing = $true
    try {
        $result = Invoke-Lethe 'status'
        if ($result.ExitCode -ne 0) {
            Set-StatusMessage (Get-PublicError $result.Error $result.ExitCode) ([System.Drawing.Color]::Firebrick)
            return
        }
        $snapshot = ConvertFrom-LetheStatus $result.Output
        Set-ControlsForState $snapshot -PreserveMessage:$PreserveMessage

        if ($script:mountPending -and $script:currentState -eq 'AVAILABLE') {
            $elapsed = [datetime]::Now - $script:pendingSince
            if ($elapsed.TotalSeconds -gt 45) {
                $script:mountPending = $false
                Set-StatusMessage 'No se abrio ningun volumen.' ([System.Drawing.Color]::DarkOrange)
            }
        }
    } finally {
        $script:refreshing = $false
    }
}

function Start-Mount {
    param(
        [Parameter(Mandatory)][ValidateSet('ro', 'rw')][string]$Mode
    )

    if ($Mode -eq 'rw') {
        $answer = [System.Windows.Forms.MessageBox]::Show(
            'La escritura modifica el volumen. Usala solamente cuando sea necesario. ¿Continuar?',
            'Confirmar escritura',
            'YesNo',
            'Warning'
        )
        if ($answer -ne 'Yes') { return }
    }

    $command = if ($Mode -eq 'ro') { 'open-ro' } else { 'open-rw' }
    $result = Invoke-Lethe $command
    if ($result.ExitCode -ne 0) {
        Set-StatusMessage (Get-PublicError $result.Error $result.ExitCode) ([System.Drawing.Color]::Firebrick)
        Refresh-LetheStatus -PreserveMessage
        return
    }

    $script:lastMode = $Mode
    $script:mountPending = $true
    $script:pendingSince = [datetime]::Now
    $writeCheck.Checked = $false
    Set-StatusMessage 'Esperando la contraseña en VeraCrypt…' ([System.Drawing.Color]::DarkOrange)
}

function Close-LetheVolume {
    $result = Invoke-Lethe 'lock'
    if ($result.ExitCode -eq 0) {
        $script:lastMode = ''
        Set-StatusMessage 'Volumen cerrado y cache limpiada.' ([System.Drawing.Color]::DarkGreen)
        Refresh-LetheStatus -PreserveMessage
        return $true
    }

    $force = [System.Windows.Forms.MessageBox]::Show(
        'El cierre normal fallo. El cierre forzado puede interrumpir archivos abiertos. ¿Intentarlo?',
        'Cierre forzado',
        'YesNo',
        'Warning'
    )
    if ($force -ne 'Yes') {
        Set-StatusMessage (Get-PublicError $result.Error $result.ExitCode) ([System.Drawing.Color]::Firebrick)
        return $false
    }

    $forcedResult = Invoke-Lethe 'force-lock'
    if ($forcedResult.ExitCode -ne 0) {
        Set-StatusMessage (Get-PublicError $forcedResult.Error $forcedResult.ExitCode) ([System.Drawing.Color]::Firebrick)
        return $false
    }
    $script:lastMode = ''
    Set-StatusMessage 'Volumen cerrado de forma forzada y cache limpiada.' ([System.Drawing.Color]::DarkOrange)
    Refresh-LetheStatus -PreserveMessage
    return $true
}

function Open-LetheFolder {
    Refresh-LetheStatus -PreserveMessage
    if ($script:currentState -ne 'MOUNTED') {
        Set-StatusMessage 'El volumen cifrado no esta abierto.' ([System.Drawing.Color]::DarkOrange)
        return
    }

    if ($script:mountLetter -notmatch '^[A-Z]$') {
        Set-StatusMessage 'Lethe recibio una letra de montaje no valida.' ([System.Drawing.Color]::Firebrick)
        return
    }

    $driveRoot = '{0}:\' -f $script:mountLetter
    if (-not (Test-LethePath $driveRoot 'Container')) {
        Set-StatusMessage "Windows aun no muestra $driveRoot. Pulsa Actualizar en unos segundos." ([System.Drawing.Color]::DarkOrange)
        return
    }

    try {
        Start-Process -FilePath 'explorer.exe' -ArgumentList $driveRoot
        Set-StatusMessage "Archivos cifrados abiertos en $driveRoot" ([System.Drawing.Color]::DarkGreen)
    } catch {
        Set-StatusMessage "No se pudo abrir $driveRoot" ([System.Drawing.Color]::Firebrick)
    }
}

function Start-VirtualRestore {
    if ($script:currentState -eq 'MOUNTED') {
        Set-StatusMessage 'Cierra el volumen antes de restaurar.' ([System.Drawing.Color]::Firebrick)
        return
    }

    $picker = [System.Windows.Forms.OpenFileDialog]::new()
    $picker.Title = 'Selecciona el disco virtual de laboratorio'
    $picker.InitialDirectory = $HomePath
    $picker.Filter = 'Discos virtuales (*.vhd;*.vhdx)|*.vhd;*.vhdx'
    $picker.CheckFileExists = $true
    $picker.Multiselect = $false
    if ($picker.ShowDialog($form) -ne 'OK') { return }

    $imagePath = [System.IO.Path]::GetFullPath($picker.FileName)
    if ($imagePath.Contains('"')) {
        Set-StatusMessage 'La ruta del disco virtual contiene caracteres no admitidos.' ([System.Drawing.Color]::Firebrick)
        return
    }
    $quotedImage = '"' + $imagePath + '"'
    $planResult = Invoke-Lethe "restore plan --virtual-image $quotedImage"
    if ($planResult.ExitCode -ne 0) {
        Set-StatusMessage (Get-PublicError $planResult.Error $planResult.ExitCode) ([System.Drawing.Color]::Firebrick)
        return
    }
    $plan = ConvertFrom-LetheStatus $planResult.Output
    foreach ($required in @('SIZE_BYTES', 'CREATION_TIME', 'LAST_WRITE_TIME', 'CONFIRMATION')) {
        if (-not $plan.ContainsKey($required)) {
            Set-StatusMessage 'El plan de restauracion esta incompleto.' ([System.Drawing.Color]::Firebrick)
            return
        }
    }
    $sizeMiB = [math]::Round(([UInt64]$plan.SIZE_BYTES / 1MB), 2)
    $answer = [System.Windows.Forms.MessageBox]::Show(
        "Esta prueba BORRARA el contenido del disco virtual:`r`n`r`n$imagePath`r`nTamano: $sizeMiB MiB`r`nResultado: exFAT / JORGITO`r`n`r`nLa memoria USB fisica no sera usada. ¿Continuar?",
        'Confirmar restauracion virtual',
        'YesNo',
        'Warning'
    )
    if ($answer -ne 'Yes') { return }

    $helperRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("LetheRestore-" + [guid]::NewGuid().ToString('N'))
    $helperBinary = Join-Path $helperRoot 'LetheRestoreHelper.exe'
    try {
        [void](New-Item -ItemType Directory -Path $helperRoot)
        Copy-Item -LiteralPath $letheBinary -Destination $helperBinary
        $arguments = "restore execute --virtual-image $quotedImage --expected-size $($plan.SIZE_BYTES) --expected-created $($plan.CREATION_TIME) --expected-write $($plan.LAST_WRITE_TIME) --confirm $($plan.CONFIRMATION)"
        Set-StatusMessage 'Esperando autorizacion de Windows…' ([System.Drawing.Color]::DarkOrange)
        $process = Start-Process -FilePath $helperBinary -ArgumentList $arguments -WorkingDirectory $HomePath -Verb RunAs -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            Set-StatusMessage 'La restauracion fue cancelada o fallo.' ([System.Drawing.Color]::Firebrick)
            return
        }
        Set-StatusMessage 'Disco virtual restaurado como exFAT.' ([System.Drawing.Color]::DarkGreen)
    } catch {
        Set-StatusMessage 'No se pudo iniciar el ayudante de restauracion.' ([System.Drawing.Color]::Firebrick)
    } finally {
        if (Test-Path -LiteralPath $helperRoot) {
            Remove-Item -LiteralPath $helperRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

function Start-PhysicalRestore {
    if ($script:currentState -eq 'MOUNTED') {
        Set-StatusMessage 'Cierra el volumen antes de restaurar.' ([System.Drawing.Color]::Firebrick)
        return
    }

    $homeRoot = [System.IO.Path]::GetPathRoot($HomePath)
    $defaultLetter = if ($homeRoot -and $homeRoot -match '^[A-Za-z]:\\$') {
        $homeRoot.Substring(0, 1).ToUpperInvariant()
    } else {
        'F'
    }
    $driveLetter = [Microsoft.VisualBasic.Interaction]::InputBox(
        'Escribe la letra actual de la memoria JORGITO, sin dos puntos.',
        'Seleccionar memoria fisica',
        $defaultLetter
    ).Trim().ToUpperInvariant()
    if (-not $driveLetter) { return }
    if ($driveLetter -notmatch '^[A-Z]$') {
        Set-StatusMessage 'La letra indicada no es valida.' ([System.Drawing.Color]::Firebrick)
        return
    }

    $planResult = Invoke-Lethe "usb restore-plan --letter $driveLetter --expected-label JORGITO"
    if ($planResult.ExitCode -ne 0) {
        Set-StatusMessage (Get-PublicError $planResult.Error $planResult.ExitCode) ([System.Drawing.Color]::Firebrick)
        return
    }
    $plan = ConvertFrom-LetheStatus $planResult.Output
    $required = @(
        'DRIVE_LETTER', 'CURRENT_LABEL', 'CURRENT_FILESYSTEM', 'DISK_NUMBER',
        'PARTITION_NUMBER', 'SIZE_BYTES', 'SERIAL', 'CONFIRMATION', 'DATA_LOSS'
    )
    foreach ($field in $required) {
        if (-not $plan.ContainsKey($field)) {
            Set-StatusMessage 'El plan fisico esta incompleto.' ([System.Drawing.Color]::Firebrick)
            return
        }
    }
    $sizeGB = [math]::Round(([UInt64]$plan.SIZE_BYTES / 1GB), 2)
    $answer = [System.Windows.Forms.MessageBox]::Show(
        "ATENCION: esta operacion eliminara TODO el contenido actual.`r`n`r`nUnidad: $($plan.DRIVE_LETTER):`r`nEtiqueta: $($plan.CURRENT_LABEL)`r`nDisco: $($plan.DISK_NUMBER)`r`nCapacidad: $sizeGB GB`r`nSerial: $($plan.SERIAL)`r`n`r`nResultado: GPT / exFAT / JORGITO`r`n`r`n¿Quieres continuar hasta la confirmacion escrita?",
        'Restauracion fisica destructiva',
        'YesNo',
        'Warning'
    )
    if ($answer -ne 'Yes') { return }

    $typed = [Microsoft.VisualBasic.Interaction]::InputBox(
        "Para autorizar el borrado escribe exactamente:`r`n$($plan.CONFIRMATION)",
        'Confirmacion final',
        ''
    )
    if ($typed -cne $plan.CONFIRMATION) {
        Set-StatusMessage 'La frase no coincide. No se hizo ningun cambio.' ([System.Drawing.Color]::Firebrick)
        return
    }

    $helperRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("LethePhysical-" + [guid]::NewGuid().ToString('N'))
    $helperBinary = Join-Path $helperRoot 'LethePhysicalRestore.exe'
    try {
        [void](New-Item -ItemType Directory -Path $helperRoot)
        Copy-Item -LiteralPath $letheBinary -Destination $helperBinary
        $arguments = "usb restore-execute --letter $($plan.DRIVE_LETTER) --expected-label $($plan.CURRENT_LABEL) --expected-disk $($plan.DISK_NUMBER) --expected-partition $($plan.PARTITION_NUMBER) --expected-size $($plan.SIZE_BYTES) --expected-filesystem $($plan.CURRENT_FILESYSTEM) --expected-serial $($plan.SERIAL) --confirm $($plan.CONFIRMATION)"
        Set-StatusMessage 'Esperando autorizacion de Windows…' ([System.Drawing.Color]::DarkOrange)
        $process = Start-Process -FilePath $helperBinary -ArgumentList $arguments -WorkingDirectory $helperRoot -Verb RunAs -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            Set-StatusMessage 'La restauracion fisica fue cancelada o fallo.' ([System.Drawing.Color]::Firebrick)
            return
        }
        [System.Windows.Forms.MessageBox]::Show(
            'La memoria fue restaurada como GPT / exFAT / JORGITO. El panel se cerrara para permitir la reconexion.',
            'Restauracion completada',
            'OK',
            'Information'
        ) | Out-Null
        $script:allowFormClose = $true
        $form.Close()
    } catch {
        Set-StatusMessage 'No se pudo iniciar el restaurador fisico.' ([System.Drawing.Color]::Firebrick)
    } finally {
        if (Test-Path -LiteralPath $helperRoot) {
            Remove-Item -LiteralPath $helperRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

$form = [System.Windows.Forms.Form]::new()
$form.Text = 'Lethe'
$form.ClientSize = [System.Drawing.Size]::new(480, 550)
$form.FormBorderStyle = 'FixedDialog'
$form.MaximizeBox = $false
$form.StartPosition = 'CenterScreen'
$form.Font = [System.Drawing.Font]::new('Segoe UI', 9)

$title = [System.Windows.Forms.Label]::new()
$title.Text = 'Lethe'
$title.Font = [System.Drawing.Font]::new('Segoe UI', 21, [System.Drawing.FontStyle]::Bold)
$title.Location = [System.Drawing.Point]::new(26, 18)
$title.AutoSize = $true
$form.Controls.Add($title)

$subtitle = [System.Windows.Forms.Label]::new()
$subtitle.Text = 'Almacenamiento cifrado'
$subtitle.Location = [System.Drawing.Point]::new(30, 63)
$subtitle.AutoSize = $true
$form.Controls.Add($subtitle)

$readOnlyButton = [System.Windows.Forms.Button]::new()
$readOnlyButton.Text = 'Abrir en modo seguro (solo lectura)'
$readOnlyButton.Location = [System.Drawing.Point]::new(30, 98)
$readOnlyButton.Size = [System.Drawing.Size]::new(420, 39)
$readOnlyButton.Add_Click({ Invoke-PanelAction { Start-Mount 'ro' } })
$form.Controls.Add($readOnlyButton)

$writeCheck = [System.Windows.Forms.CheckBox]::new()
$writeCheck.Text = 'Habilitar la opcion de escritura'
$writeCheck.Location = [System.Drawing.Point]::new(32, 149)
$writeCheck.Size = [System.Drawing.Size]::new(350, 24)
$writeCheck.Add_CheckedChanged({
    $writeButton.Enabled = $writeCheck.Checked -and $script:currentState -eq 'AVAILABLE'
})
$form.Controls.Add($writeCheck)

$writeButton = [System.Windows.Forms.Button]::new()
$writeButton.Text = 'Abrir con escritura'
$writeButton.Location = [System.Drawing.Point]::new(30, 179)
$writeButton.Size = [System.Drawing.Size]::new(204, 37)
$writeButton.Enabled = $false
$writeButton.Add_Click({ Invoke-PanelAction { Start-Mount 'rw' } })
$form.Controls.Add($writeButton)

$closeButton = [System.Windows.Forms.Button]::new()
$closeButton.Text = 'Desmontar volumen'
$closeButton.Location = [System.Drawing.Point]::new(246, 179)
$closeButton.Size = [System.Drawing.Size]::new(204, 37)
$closeButton.Enabled = $false
$closeButton.Add_Click({ Invoke-PanelAction { [void](Close-LetheVolume) } })
$form.Controls.Add($closeButton)

$restoreButton = [System.Windows.Forms.Button]::new()
$restoreButton.Text = 'Restaurar disco virtual de prueba'
$restoreButton.Location = [System.Drawing.Point]::new(30, 228)
$restoreButton.Size = [System.Drawing.Size]::new(420, 34)
$restoreButton.Enabled = $true
$restoreButton.Add_Click({ Invoke-PanelAction { Start-VirtualRestore } })
$form.Controls.Add($restoreButton)

$physicalRestoreButton = [System.Windows.Forms.Button]::new()
$physicalRestoreButton.Text = 'Restaurar memoria fisica'
$physicalRestoreButton.Location = [System.Drawing.Point]::new(30, 270)
$physicalRestoreButton.Size = [System.Drawing.Size]::new(420, 34)
$physicalRestoreButton.Enabled = $true
$physicalRestoreButton.Add_Click({ Invoke-PanelAction { Start-PhysicalRestore } })
$form.Controls.Add($physicalRestoreButton)

$refreshButton = [System.Windows.Forms.Button]::new()
$refreshButton.Text = 'Actualizar estado'
$refreshButton.Location = [System.Drawing.Point]::new(30, 316)
$refreshButton.Size = [System.Drawing.Size]::new(204, 34)
$refreshButton.Add_Click({ Invoke-PanelAction { Refresh-LetheStatus } })
$form.Controls.Add($refreshButton)

$openFolderButton = [System.Windows.Forms.Button]::new()
$openFolderButton.Text = 'Abrir archivos cifrados'
$openFolderButton.Location = [System.Drawing.Point]::new(246, 316)
$openFolderButton.Size = [System.Drawing.Size]::new(204, 34)
$openFolderButton.Enabled = $false
$openFolderButton.Add_Click({ Invoke-PanelAction { Open-LetheFolder } })
$form.Controls.Add($openFolderButton)

$statusLabel = [System.Windows.Forms.Label]::new()
$statusLabel.Text = 'Comprobando Lethe…'
$statusLabel.Location = [System.Drawing.Point]::new(31, 364)
$statusLabel.Size = [System.Drawing.Size]::new(418, 42)
$statusLabel.Font = [System.Drawing.Font]::new('Segoe UI', 9, [System.Drawing.FontStyle]::Bold)
$form.Controls.Add($statusLabel)

$diagnosticGroup = [System.Windows.Forms.GroupBox]::new()
$diagnosticGroup.Text = 'Diagnostico'
$diagnosticGroup.Location = [System.Drawing.Point]::new(30, 409)
$diagnosticGroup.Size = [System.Drawing.Size]::new(420, 115)
$form.Controls.Add($diagnosticGroup)

$diagnosticBox = [System.Windows.Forms.TextBox]::new()
$diagnosticBox.Location = [System.Drawing.Point]::new(10, 21)
$diagnosticBox.Size = [System.Drawing.Size]::new(400, 84)
$diagnosticBox.Multiline = $true
$diagnosticBox.ReadOnly = $true
$diagnosticBox.BorderStyle = 'None'
$diagnosticBox.BackColor = $form.BackColor
$diagnosticGroup.Controls.Add($diagnosticBox)

$timer = [System.Windows.Forms.Timer]::new()
$timer.Interval = 2500
$timer.Add_Tick({ Invoke-PanelAction { Refresh-LetheStatus -PreserveMessage:$script:mountPending } })

$form.Add_Shown({
    Invoke-PanelAction {
        Refresh-LetheStatus
        $timer.Start()
    }
})

$form.Add_FormClosing({
    param($sender, $eventArgs)
    if ($script:allowFormClose -or $script:currentState -ne 'MOUNTED') { return }

    $answer = [System.Windows.Forms.MessageBox]::Show(
        'Hay un volumen abierto. ¿Quieres cerrarlo antes de salir?',
        'Cerrar Lethe',
        'YesNoCancel',
        'Question'
    )
    if ($answer -eq 'Cancel') {
        $eventArgs.Cancel = $true
    } elseif ($answer -eq 'Yes') {
        if (-not (Close-LetheVolume)) {
            $eventArgs.Cancel = $true
        } else {
            $script:allowFormClose = $true
        }
    } else {
        $script:allowFormClose = $true
    }
})

[void]$form.ShowDialog()
$timer.Stop()
$timer.Dispose()
