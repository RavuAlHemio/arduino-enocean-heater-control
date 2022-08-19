Third-party files have been obtained from the following sources:

* `ATSAM3X8E.svd`: From the _Atmel SAM3X Series Device Support_ package, available for download at
  http://packs.download.atmel.com/ (notably _not_ https://packs.download.microchip.com/).

To regenerate the source files, `cargo install svd2rust` and then run:

    svd2rust -i ATSAM3X8E.svd -o . --target cortex-m
    mv lib.rs src
