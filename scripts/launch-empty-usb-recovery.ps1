Add-Type -AssemblyName System.Windows.Forms
$answer = [System.Windows.Forms.MessageBox]::Show(
    "Lethe verifico que el disco USB 2, serial 057907B76060, esta saludable pero sin particiones.`r`n`r`nLa reparacion creara GPT, exFAT y la etiqueta JORGITO sin tocar otro disco. ¿Continuar?",
    'Reparar restauracion interrumpida',
    'YesNo',
    'Warning'
)
if ($answer -ne 'Yes') { exit 2 }

$recoveryScript = Join-Path $PSScriptRoot 'recover-empty-usb.ps1'
$arguments = @(
    '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
    '-File', ('"' + $recoveryScript + '"'),
    '-ExpectedDiskNumber', '2',
    '-ExpectedSize', '8011120640',
    '-ExpectedSerial', '057907B76060',
    '-ResultLabel', 'JORGITO'
) -join ' '
$process = Start-Process -FilePath 'powershell.exe' -ArgumentList $arguments -WorkingDirectory $PSScriptRoot -Verb RunAs -Wait -PassThru
if ($process.ExitCode -ne 0) {
    [System.Windows.Forms.MessageBox]::Show(
        "La reparacion no termino correctamente. Codigo: $($process.ExitCode)",
        'Lethe', 'OK', 'Error'
    ) | Out-Null
    exit $process.ExitCode
}
[System.Windows.Forms.MessageBox]::Show(
    'La estructura GPT/exFAT fue creada. Lethe verificara ahora el resultado.',
    'Lethe', 'OK', 'Information'
) | Out-Null
