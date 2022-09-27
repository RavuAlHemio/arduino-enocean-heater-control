$downloadDir = (New-Object -ComObject "Shell.Application").NameSpace("shell:Downloads").Self.Path
& cargo run --bin eepxml2rust --release "$downloadDir\enocean-eep268.xml" ..\buildingblocks\src\esp3\eep.rs
