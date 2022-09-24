[CmdletBinding()]
Param (
    [Parameter(Position=0)]
    [string]
    $ComPort,

    [Parameter(Position=1)]
    [string]
    $DebugPort,

    [switch]
    $NoBuild
)


If (-not $NoBuild)
{
    & cargo build --release
    If ($LASTEXITCODE -ne 0)
    {
        Return 1
    }

    & rust-objcopy --output-target=binary .\target\thumbv7m-none-eabi\release\arduino_enocean_heater_control .\aehc
    If ($LASTEXITCODE -ne 0)
    {
        Return 1
    }
}

$kilobytes = (Get-Item -LiteralPath .\aehc).Length / 1024
Write-Output ("{0:#,##0.###}" -f $kilobytes)

If ($ComPort -ne "")
{
    & 'C:\Program Files (x86)\BOSSA\bossac.exe' --arduino-erase --erase --write --boot=1 --port=$ComPort .\aehc
}

If ($DebugPort -ne "")
{
    & 'C:\Program Files\arm-gcc\bin\arm-none-eabi-gdb.exe' `
        -ex "target extended-remote \\.\$DebugPort" `
        -ex "monitor jtag_scan" `
        -ex "attach 1" `
        .\target\thumbv7m-none-eabi\release\arduino_enocean_heater_control
}
