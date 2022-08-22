[CmdletBinding()]
Param (
    [Parameter(Mandatory=$true, Position=0)]
    $ComPort
)


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

& 'C:\Program Files (x86)\BOSSA\bossac.exe' --arduino-erase --erase --write --boot=1 --port=$ComPort .\aehc
