[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z]$')]
    [string]$DriveLetter
)

$ErrorActionPreference = 'Stop'
$volume = Get-Volume -DriveLetter $DriveLetter
$partitions = @(Get-Partition -DriveLetter $DriveLetter)
if ($partitions.Count -ne 1) {
    throw "La letra indicada no corresponde a una unica particion."
}
$partition = $partitions[0]
$disk = $partition | Get-Disk
if ($null -eq $disk -or @($disk).Count -ne 1) {
    throw "No se pudo resolver un unico disco desde la letra indicada."
}

Write-Output "LABEL=$($volume.FileSystemLabel)"
Write-Output "FILESYSTEM=$($volume.FileSystem)"
Write-Output "DISK_NUMBER=$($disk.Number)"
Write-Output "PARTITION_NUMBER=$($partition.PartitionNumber)"
Write-Output "PARTITION_COUNT=$($partitions.Count)"
Write-Output "SIZE_BYTES=$($disk.Size)"
Write-Output "BUS_TYPE=$($disk.BusType)"
Write-Output "IS_BOOT=$([int][bool]$disk.IsBoot)"
Write-Output "IS_SYSTEM=$([int][bool]$disk.IsSystem)"
Write-Output "IS_READ_ONLY=$([int][bool]$disk.IsReadOnly)"
Write-Output "IS_OFFLINE=$([int][bool]$disk.IsOffline)"
Write-Output "SERIAL=$($disk.SerialNumber)"
