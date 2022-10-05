use atsam3x8e::Peripherals;
use atsam3x8e::pmc::ckgr_mor::KEY_A::PASSWD;
use atsam3x8e::pmc::ckgr_mor::MOSCRCF_A;
use atsam3x8e::pmc::pmc_mckr::CSS_A;
use atsam3x8e::pmc::pmc_mckr::PRES_A::CLK_2;


// BEGIN SAM3X8E-specific properties
const SYS_BOARD_OSCOUNT: u8 = 0x8;

#[inline(always)]
fn set_sys_board_pllar(peripherals: &mut Peripherals) {
    unsafe {
        peripherals.PMC.ckgr_pllar.write_with_zero(|w| w
            .one().set_bit()
            .mula().variant(0xD)
            .pllacount().variant(0x3F)
            .diva().variant(0x1)
        )
    }
}

#[inline(always)]
fn set_sys_board_mkcr(peripherals: &mut Peripherals) {
    unsafe {
        peripherals.PMC.pmc_mckr.write_with_zero(|w| w
            .pres().variant(CLK_2)
            .css().variant(CSS_A::PLLA_CLK)
        )
    }
}

#[inline(always)]
fn set_sys_board_mkcr_with_custom_clock(peripherals: &mut Peripherals, custom_clock: CSS_A) {
    unsafe {
        peripherals.PMC.pmc_mckr.write_with_zero(|w| w
            .pres().variant(CLK_2)
            .css().variant(custom_clock)
        )
    }
}

const CHIP_FREQ_SLCK_RC_MIN: u32 = 20000;
const CHIP_FREQ_SLCK_RC: u32 = 32000;
const CHIP_FREQ_SLCK_RC_MAX: u32 = 44000;
const CHIP_FREQ_MAINCK_RC_4MHZ: u32 = 4000000;
const CHIP_FREQ_MAINCK_RC_8MHZ: u32 = 8000000;
const CHIP_FREQ_MAINCK_RC_12MHZ: u32 = 12000000;
pub const CHIP_FREQ_CPU_MAX: u32 = 84000000;
const CHIP_FREQ_XTAL_32K: u32 = 32768;
const CHIP_FREQ_XTAL_12M: u32 = 12000000;
/// UTMI PLL frequency
const CHIP_FREQ_UTMIPLL: u32 = 480000000;

// END SAM3X8E-specific properties


pub struct SystemCoreClock {
    pub clock_speed: u32,
}
impl SystemCoreClock {
    pub fn new() -> Self {
        Self {
            clock_speed: CHIP_FREQ_MAINCK_RC_4MHZ,
        }
    }

    /// Update the clock frequency according to clock register values
    pub fn update(&mut self, peripherals: &mut Peripherals) {
        let clock_mode = peripherals.PMC.pmc_mckr.read().css().variant();
        if clock_mode == CSS_A::SLOW_CLK {
            if peripherals.SUPC.sr.read().oscsel().bit_is_set() {
                self.clock_speed = CHIP_FREQ_XTAL_32K;
            } else {
                self.clock_speed = CHIP_FREQ_SLCK_RC;
            }
            return;
        }

        // common code for MAIN, PLLA and UPLL clocks
        if peripherals.PMC.ckgr_mor.read().moscsel().bit_is_set() {
            self.clock_speed = CHIP_FREQ_XTAL_12M;
        } else {
            self.clock_speed = CHIP_FREQ_MAINCK_RC_4MHZ;

            match peripherals.PMC.ckgr_mor.read().moscrcf().variant() {
                Some(MOSCRCF_A::_4_MHZ) => {},
                Some(MOSCRCF_A::_8_MHZ) => {
                    self.clock_speed *= 2;
                },
                Some(MOSCRCF_A::_12_MHZ) => {
                    self.clock_speed *= 3;
                },
                _ => {},
            }
        }

        // addenda for PLLA and UPLL
        if clock_mode == CSS_A::PLLA_CLK {
            let pllar = peripherals.PMC.ckgr_pllar.read();
            self.clock_speed *= u32::from(pllar.mula().bits()) + 1;
            self.clock_speed /= u32::from(pllar.diva().bits());
        } else if clock_mode == CSS_A::UPLL_CLK {
            self.clock_speed = CHIP_FREQ_UTMIPLL / 2;
        }

        // final modifications
        let mckr = peripherals.PMC.pmc_mckr.read();
        if mckr.pres().is_clk_3() {
            self.clock_speed /= 3;
        } else {
            self.clock_speed >>= mckr.pres().bits();
        }
    }
}


pub fn system_init(peripherals: &mut Peripherals) -> SystemCoreClock {
    // translated from SystemInit() in gcc/system_sam3xa.c
    // from Atmel SAM3X Series Device Support package

    // set FWS according to SYS_BOARD_MKCR configuration
    peripherals.EFC0.fmr.write(|w| w.fws().variant(4));
    peripherals.EFC1.fmr.write(|w| w.fws().variant(4));

    // initialize main oscillator
    if peripherals.PMC.ckgr_mor.read().moscsel().bit_is_clear() {
        unsafe {
            peripherals.PMC.ckgr_mor.write_with_zero(|w| w
                .key().variant(PASSWD)
                .moscxtst().variant(SYS_BOARD_OSCOUNT)
                .moscrcen().set_bit()
                .moscxten().set_bit()
            )
        };

        while peripherals.PMC.pmc_sr.read().moscxts().bit_is_clear() {
            // wait
        }
    }

    // switch to crystal oscillator
    unsafe {
        peripherals.PMC.ckgr_mor.write_with_zero(|w| w
            .key().variant(PASSWD)
            .moscxtst().variant(SYS_BOARD_OSCOUNT)
            .moscrcen().set_bit()
            .moscxten().set_bit()
            .moscsel().set_bit()
        )
    };
    while peripherals.PMC.pmc_sr.read().moscsels().bit_is_clear() {
        // wait
    }

    peripherals.PMC.pmc_mckr.modify(|_, w| w
        .css().variant(CSS_A::MAIN_CLK)
    );
    while peripherals.PMC.pmc_sr.read().mckrdy().bit_is_clear() {
        // wait
    }

    // initialize PLLA
    set_sys_board_pllar(peripherals);
    while peripherals.PMC.pmc_sr.read().locka().bit_is_clear() {
        // wait
    }

    // switch to main clock
    set_sys_board_mkcr_with_custom_clock(peripherals, CSS_A::MAIN_CLK);
    while peripherals.PMC.pmc_sr.read().mckrdy().bit_is_clear() {
        // wait
    }

    // switch to PLLA
    set_sys_board_mkcr(peripherals);
    while peripherals.PMC.pmc_sr.read().mckrdy().bit_is_clear() {
        // wait
    }

    SystemCoreClock {
        clock_speed: CHIP_FREQ_CPU_MAX,
    }
}
