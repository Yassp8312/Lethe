[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][UInt32]$ExpectedDiskNumber,
    [Parameter(Mandatory = $true)][UInt64]$ExpectedSize,
    [Parameter(Mandatory = $true)][string]$ExpectedSerial,
    [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z0-9_-]{1,15}$')][string]$ResultLabel
)

$ErrorActionPreference = 'Stop'
$mountRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("LetheRecoveryMount-" + [guid]::NewGuid().ToString('N'))
$accessPath = $mountRoot + [System.IO.Path]::DirectorySeparatorChar
$newPartition = $null

try {
    $disk = Get-Disk -Number $ExpectedDiskNumber
    if (($disk.SerialNumber.Trim()) -cne $ExpectedSerial.Trim()) { throw 'El serial no coincide.' }
    if ([UInt64]$disk.Size -ne $ExpectedSize) { throw 'La capacidad no coincide.' }
    if ($disk.BusType -ne 'USB') { throw 'El objetivo no usa bus USB.' }
    if ($disk.IsBoot -or $disk.IsSystem) { throw 'El objetivo pertenece al arranque o al sistema.' }
    if ($disk.IsReadOnly -or $disk.IsOffline) { throw 'El objetivo no esta disponible para escritura.' }
    $partitions = @(Get-Partition -DiskNumber $ExpectedDiskNumber -ErrorAction SilentlyContinue)
    if ($partitions.Count -ne 0) { throw 'La recuperacion solo admite el disco vacio de la operacion interrumpida.' }

    New-Item -ItemType Directory -Path $mountRoot | Out-Null
    if ($disk.PartitionStyle -eq 'RAW') {
        Initialize-Disk -Number $ExpectedDiskNumber -PartitionStyle GPT | Out-Null
    } elseif ($disk.PartitionStyle -ne 'GPT') {
        Set-Disk -Number $ExpectedDiskNumber -PartitionStyle GPT
    }
    $newPartition = New-Partition -DiskNumber $ExpectedDiskNumber -UseMaximumSize
    Add-PartitionAccessPath -DiskNumber $ExpectedDiskNumber -PartitionNumber $newPartition.PartitionNumber -AccessPath $accessPath
    $newPartition | Format-Volume -FileSystem exFAT -NewFileSystemLabel $ResultLabel -Confirm:$false -Force | Out-Null

    $volume = $newPartition | Get-Volume
    $verifiedDisk = Get-Disk -Number $ExpectedDiskNumber
    if ($verifiedDisk.PartitionStyle -ne 'GPT') { throw 'No se verifico GPT.' }
    if ($volume.FileSystem -ne 'exFAT') { throw 'No se verifico exFAT.' }
    if ($volume.FileSystemLabel -cne $ResultLabel) { throw 'No se verifico la etiqueta.' }
    Write-Output 'RECOVERY_OK=1'
} finally {
    if ($null -ne $newPartition) {
        Remove-PartitionAccessPath -DiskNumber $ExpectedDiskNumber -PartitionNumber $newPartition.PartitionNumber -AccessPath $accessPath -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $mountRoot) {
        Remove-Item -LiteralPath $mountRoot -Force -ErrorAction SilentlyContinue
    }
}
