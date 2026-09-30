[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z]$')][string]$DriveLetter,
    [Parameter(Mandatory = $true)][string]$ExpectedLabel,
    [Parameter(Mandatory = $true)][string]$ExpectedFileSystem,
    [Parameter(Mandatory = $true)][UInt32]$ExpectedDiskNumber,
    [Parameter(Mandatory = $true)][UInt32]$ExpectedPartitionNumber,
    [Parameter(Mandatory = $true)][UInt64]$ExpectedSize,
    [Parameter(Mandatory = $true)][string]$ExpectedSerial,
    [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z0-9_-]{1,15}$')][string]$ResultLabel
)

$ErrorActionPreference = 'Stop'
$mountRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("LethePhysicalMount-" + [guid]::NewGuid().ToString('N'))
$accessPath = $mountRoot + [System.IO.Path]::DirectorySeparatorChar
$mountedPartition = $null

function Resolve-ExactTarget {
    $volume = Get-Volume -DriveLetter $DriveLetter
    $partitions = @(Get-Partition -DriveLetter $DriveLetter)
    if ($partitions.Count -ne 1) { throw 'La letra no corresponde a una unica particion.' }
    $partition = $partitions[0]
    $disk = $partition | Get-Disk
    if ($null -eq $disk -or @($disk).Count -ne 1) { throw 'La letra no resuelve un unico disco.' }
    $allPartitions = @(Get-Partition -DiskNumber $disk.Number)

    if ($volume.FileSystemLabel -cne $ExpectedLabel) { throw 'La etiqueta cambio despues del plan.' }
    if ($volume.FileSystem -cne $ExpectedFileSystem) { throw 'El sistema de archivos cambio despues del plan.' }
    if ([UInt32]$disk.Number -ne $ExpectedDiskNumber) { throw 'El numero de disco cambio despues del plan.' }
    if ([UInt32]$partition.PartitionNumber -ne $ExpectedPartitionNumber) { throw 'La particion cambio despues del plan.' }
    if ($allPartitions.Count -ne 1) { throw 'El numero de particiones cambio despues del plan.' }
    if ([UInt64]$disk.Size -ne $ExpectedSize) { throw 'La capacidad cambio despues del plan.' }
    if (($disk.SerialNumber.Trim()) -cne $ExpectedSerial.Trim()) { throw 'El serial cambio despues del plan.' }
    if ($disk.BusType -ne 'USB') { throw 'El bus del objetivo no es USB.' }
    if ($disk.IsBoot -or $disk.IsSystem) { throw 'El objetivo pertenece al arranque o al sistema.' }
    if ($disk.IsReadOnly) { throw 'El objetivo esta protegido contra escritura.' }
    if ($disk.IsOffline) { throw 'El objetivo esta fuera de linea.' }

    return [pscustomobject]@{ Volume = $volume; Partition = $partition; Disk = $disk }
}

try {
    $target = Resolve-ExactTarget
    New-Item -ItemType Directory -Path $mountRoot | Out-Null

    # Esta es la primera instruccion destructiva. Todas las comprobaciones anteriores deben pasar.
    Clear-Disk -Number $target.Disk.Number -RemoveData -RemoveOEM -Confirm:$false
    $clearedDisk = Get-Disk -Number $target.Disk.Number
    if ($clearedDisk.PartitionStyle -eq 'RAW') {
        Initialize-Disk -Number $target.Disk.Number -PartitionStyle GPT | Out-Null
    } elseif ($clearedDisk.PartitionStyle -ne 'GPT') {
        Set-Disk -Number $target.Disk.Number -PartitionStyle GPT
    }
    $mountedPartition = New-Partition -DiskNumber $target.Disk.Number -UseMaximumSize
    Add-PartitionAccessPath -DiskNumber $target.Disk.Number -PartitionNumber $mountedPartition.PartitionNumber -AccessPath $accessPath
    $mountedPartition | Format-Volume -FileSystem exFAT -NewFileSystemLabel $ResultLabel -Confirm:$false -Force | Out-Null

    $resultVolume = Get-Volume -FileSystemLabel $ResultLabel | Where-Object {
        $_.Path -eq $mountedPartition.AccessPaths[0] -or $_.UniqueId -eq (($mountedPartition | Get-Volume).UniqueId)
    } | Select-Object -First 1
    if ($null -eq $resultVolume) { $resultVolume = $mountedPartition | Get-Volume }
    $resultDisk = $mountedPartition | Get-Disk
    if ($resultVolume.FileSystem -ne 'exFAT') { throw 'La verificacion final no encontro exFAT.' }
    if ($resultVolume.FileSystemLabel -cne $ResultLabel) { throw 'La verificacion final no encontro la etiqueta esperada.' }
    if ($resultDisk.PartitionStyle -ne 'GPT') { throw 'La verificacion final no encontro GPT.' }
    if ([UInt32]$resultDisk.Number -ne $ExpectedDiskNumber) { throw 'El disco cambio durante la restauracion.' }

    Write-Output 'PHYSICAL_RESTORE_OK=1'
    Write-Output "DISK_NUMBER=$($resultDisk.Number)"
    Write-Output 'PARTITION_STYLE=GPT'
    Write-Output 'FILESYSTEM=exFAT'
    Write-Output "LABEL=$ResultLabel"
} finally {
    if ($null -ne $mountedPartition) {
        Remove-PartitionAccessPath -DiskNumber $mountedPartition.DiskNumber -PartitionNumber $mountedPartition.PartitionNumber -AccessPath $accessPath -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $mountRoot) {
        Remove-Item -LiteralPath $mountRoot -Force -ErrorAction SilentlyContinue
    }
}
