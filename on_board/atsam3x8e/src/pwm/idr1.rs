#[doc = "Register `IDR1` writer"]
pub struct W(crate::W<IDR1_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<IDR1_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<IDR1_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<IDR1_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `CHID0` writer - Counter Event on Channel 0 Interrupt Disable"]
pub type CHID0_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID1` writer - Counter Event on Channel 1 Interrupt Disable"]
pub type CHID1_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID2` writer - Counter Event on Channel 2 Interrupt Disable"]
pub type CHID2_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID3` writer - Counter Event on Channel 3 Interrupt Disable"]
pub type CHID3_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID4` writer - Counter Event on Channel 4 Interrupt Disable"]
pub type CHID4_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID5` writer - Counter Event on Channel 5 Interrupt Disable"]
pub type CHID5_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID6` writer - Counter Event on Channel 6 Interrupt Disable"]
pub type CHID6_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `CHID7` writer - Counter Event on Channel 7 Interrupt Disable"]
pub type CHID7_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID0` writer - Fault Protection Trigger on Channel 0 Interrupt Disable"]
pub type FCHID0_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID1` writer - Fault Protection Trigger on Channel 1 Interrupt Disable"]
pub type FCHID1_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID2` writer - Fault Protection Trigger on Channel 2 Interrupt Disable"]
pub type FCHID2_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID3` writer - Fault Protection Trigger on Channel 3 Interrupt Disable"]
pub type FCHID3_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID4` writer - Fault Protection Trigger on Channel 4 Interrupt Disable"]
pub type FCHID4_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID5` writer - Fault Protection Trigger on Channel 5 Interrupt Disable"]
pub type FCHID5_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID6` writer - Fault Protection Trigger on Channel 6 Interrupt Disable"]
pub type FCHID6_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
#[doc = "Field `FCHID7` writer - Fault Protection Trigger on Channel 7 Interrupt Disable"]
pub type FCHID7_W<'a, const O: u8> = crate::BitWriter<'a, u32, IDR1_SPEC, bool, O>;
impl W {
    #[doc = "Bit 0 - Counter Event on Channel 0 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid0(&mut self) -> CHID0_W<0> {
        CHID0_W::new(self)
    }
    #[doc = "Bit 1 - Counter Event on Channel 1 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid1(&mut self) -> CHID1_W<1> {
        CHID1_W::new(self)
    }
    #[doc = "Bit 2 - Counter Event on Channel 2 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid2(&mut self) -> CHID2_W<2> {
        CHID2_W::new(self)
    }
    #[doc = "Bit 3 - Counter Event on Channel 3 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid3(&mut self) -> CHID3_W<3> {
        CHID3_W::new(self)
    }
    #[doc = "Bit 4 - Counter Event on Channel 4 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid4(&mut self) -> CHID4_W<4> {
        CHID4_W::new(self)
    }
    #[doc = "Bit 5 - Counter Event on Channel 5 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid5(&mut self) -> CHID5_W<5> {
        CHID5_W::new(self)
    }
    #[doc = "Bit 6 - Counter Event on Channel 6 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid6(&mut self) -> CHID6_W<6> {
        CHID6_W::new(self)
    }
    #[doc = "Bit 7 - Counter Event on Channel 7 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn chid7(&mut self) -> CHID7_W<7> {
        CHID7_W::new(self)
    }
    #[doc = "Bit 16 - Fault Protection Trigger on Channel 0 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid0(&mut self) -> FCHID0_W<16> {
        FCHID0_W::new(self)
    }
    #[doc = "Bit 17 - Fault Protection Trigger on Channel 1 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid1(&mut self) -> FCHID1_W<17> {
        FCHID1_W::new(self)
    }
    #[doc = "Bit 18 - Fault Protection Trigger on Channel 2 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid2(&mut self) -> FCHID2_W<18> {
        FCHID2_W::new(self)
    }
    #[doc = "Bit 19 - Fault Protection Trigger on Channel 3 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid3(&mut self) -> FCHID3_W<19> {
        FCHID3_W::new(self)
    }
    #[doc = "Bit 20 - Fault Protection Trigger on Channel 4 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid4(&mut self) -> FCHID4_W<20> {
        FCHID4_W::new(self)
    }
    #[doc = "Bit 21 - Fault Protection Trigger on Channel 5 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid5(&mut self) -> FCHID5_W<21> {
        FCHID5_W::new(self)
    }
    #[doc = "Bit 22 - Fault Protection Trigger on Channel 6 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid6(&mut self) -> FCHID6_W<22> {
        FCHID6_W::new(self)
    }
    #[doc = "Bit 23 - Fault Protection Trigger on Channel 7 Interrupt Disable"]
    #[inline(always)]
    #[must_use]
    pub fn fchid7(&mut self) -> FCHID7_W<23> {
        FCHID7_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Interrupt Disable Register 1\n\nThis register you can [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [idr1](index.html) module"]
pub struct IDR1_SPEC;
impl crate::RegisterSpec for IDR1_SPEC {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [idr1::W](W) writer structure"]
impl crate::Writable for IDR1_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
