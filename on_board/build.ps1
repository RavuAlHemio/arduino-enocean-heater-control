[CmdletBinding()]
Param (
    [Parameter(Position=0)]
    [string]
    $ComPort,

    [Parameter(Position=1)]
    [string]
    $DebugPort,

    [switch]
    $NoBuild,

    [string]
    $BinaryName="arduino_enocean_heater_control",

    [switch]
    $ProgramOOCD,

    [switch]
    $DebugBuild,

    [switch]
    $DebugOOCD
)


$targetSubdir = If ($DebugBuild) { "debug" } Else { "release" }


If (-not $NoBuild)
{
    If ($DebugBuild)
    {
        & cargo build
    }
    Else
    {
        & cargo build --release
    }

    If ($LASTEXITCODE -ne 0)
    {
        Return 1
    }

    & rust-objcopy --output-target=binary ".\target\thumbv7m-none-eabi\$targetSubdir\$BinaryName" ".\$BinaryName.bin"
    If ($LASTEXITCODE -ne 0)
    {
        Return 1
    }
}

$kilobytes = (Get-Item -LiteralPath ".\$BinaryName.bin").Length / 1024
Write-Output ("{0:#,##0.###} KiB" -f $kilobytes)

If ($ProgramOOCD)
{
    & "C:\Program Files\OpenOCD\bin\openocd.exe" `
        -c "set BINFILE $BinaryName.bin" `
        -c "source oocd-prog-jlink.cfg"
}
ElseIf ($ComPort -ne "")
{
    # disable DTR on the COM port before running BOSSA
    & '..\on_host\target\debug\winsersetup.exe' --disable-dtr $ComPort
    & 'C:\Program Files (x86)\BOSSA\bossac.exe' --arduino-erase --erase --write --boot=1 --port=$ComPort ".\$BinaryName.bin"
}

If ($DebugOOCD)
{
    $oocd = Start-Process `
        -FilePath "pwsh.exe" `
        -ArgumentList @("-NoExit -Command & \`"C:\Program Files\OpenOCD\bin\openocd.exe\`" -c \`"source oocd-debug-jlink.cfg\`"") `
        -PassThru

    $gdb = Start-Process `
        -FilePath 'C:\Program Files\arm-gcc\bin\arm-none-eabi-gdb.exe' `
        -ArgumentList @("`"-ex`" `"target extended-remote :3333`" `".\target\thumbv7m-none-eabi\$targetSubdir\$BinaryName`"") `
        -PassThru

    Write-Output "Exit GDB and OpenOCD to return to console."
    $oocd.WaitForExit()
    $gdb.WaitForExit()
}
ElseIf ($DebugPort -ne "")
{
    & 'C:\Program Files\arm-gcc\bin\arm-none-eabi-gdb.exe' `
        -ex "target extended-remote \\.\$DebugPort" `
        -ex "monitor jtag_scan" `
        -ex "attach 1" `
        ".\target\thumbv7m-none-eabi\$targetSubdir\$BinaryName"
}
