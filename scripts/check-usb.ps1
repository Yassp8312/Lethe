[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [ValidatePattern('^[A-Za-z]$')]
    [string]$DriveLetter = 'E',

    [Parameter(Mandatory = $false)]
    [string]$ExpectedLabel = 'JORGITO'
)

$ErrorActionPreference = 'Stop'
$volume = Get-Volume -DriveLetter $DriveLetter
$partition = Get-Partition -DriveLetter $DriveLetter
$disk = $partition | Get-Disk

$result = [pscustomobject]@{
    DriveLetter    = $volume.DriveLetter
    Label          = $volume.FileSystemLabel
    FileSystem     = $volume.FileSystem
    SizeGB         = [math]::Round($volume.Size / 1GB, 2)
    FreeGB         = [math]::Round($volume.SizeRemaining / 1GB, 2)
    DiskNumber     = $disk.Number
    FriendlyName   = $disk.FriendlyName
    BusType        = $disk.BusType
    PartitionStyle = $disk.PartitionStyle
    IsBoot         = $disk.IsBoot
    IsSystem       = $disk.IsSystem
}

$result | Format-List

if ($disk.BusType -ne 'USB') {
    throw "La unidad $DriveLetter`: no pertenece a un disco USB."
}
if ($disk.IsBoot -or $disk.IsSystem) {
    throw 'La unidad seleccionada pertenece al disco de arranque o del sistema.'
}
if ($volume.FileSystemLabel -ne $ExpectedLabel) {
    throw "Etiqueta inesperada: '$($volume.FileSystemLabel)'. Se esperaba '$ExpectedLabel'."
}

Write-Host 'Validacion superada. Este script no modifica el dispositivo.' -ForegroundColor Green

