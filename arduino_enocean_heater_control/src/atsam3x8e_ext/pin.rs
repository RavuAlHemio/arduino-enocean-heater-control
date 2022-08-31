#[macro_export]
macro_rules! sam_pin {
    (enable_io, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // enable I/O (disabling peripheral access)
        unsafe {
            $peripherals.$pio.per.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        };
    };
    (disable_io, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // disable I/O (enabling peripheral access)
        unsafe {
            $peripherals.$pio.pdr.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        };
    };
    (peripheral_ab, $peripherals:expr, $pio:ident, $($pin:ident, $pin_set_clear:ident),+) => {
        // when I/O is disabled, selects between peripherals A (clear) and B (set)
        //
        // warning: this is not a mailbox register; read-modify-write it
        unsafe {
            $peripherals.$pio.absr.modify(|_, w| w
                $(.$pin().$pin_set_clear())*
            )
        };
    };
    (make_output, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // make pins outputs
        unsafe {
            $peripherals.$pio.oer.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (make_input, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // make pins inputs (disable output)
        unsafe {
            $peripherals.$pio.odr.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (set_high, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // set pins high
        unsafe {
            $peripherals.$pio.sodr.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (set_low, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // set pins low
        unsafe {
            $peripherals.$pio.codr.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (enable_pullup, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // enable pullup on input pins
        unsafe {
            $peripherals.$pio.puer.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (disable_pullup, $peripherals:expr, $pio:ident, $($pin:ident),+) => {
        // disable pullup on input pins
        unsafe {
            $peripherals.$pio.pudr.write_with_zero(|w| w
                $(.$pin().set_bit())*
            )
        }
    };
    (is_up, $peripherals:expr, $pio:ident, $pin:ident) => {
        $peripherals.$pio.pdsr.read().$pin().bit_is_set()
    };
    (is_down, $peripherals:expr, $pio:ident, $pin:ident) => {
        $peripherals.$pio.pdsr.read().$pin().bit_is_clear()
    };
}
