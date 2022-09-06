#[doc = "Register `DTUPD6` writer"]
pub struct W(crate::W<DTUPD6_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<DTUPD6_SPEC>;
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
impl From<crate::W<DTUPD6_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<DTUPD6_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `DTHUPD` writer - Dead-Time Value Update for PWMHx Output"]
pub type DTHUPD_W<'a, const O: u8> = crate::FieldWriter<'a, u32, DTUPD6_SPEC, u16, u16, 16, O>;
#[doc = "Field `DTLUPD` writer - Dead-Time Value Update for PWMLx Output"]
pub type DTLUPD_W<'a, const O: u8> = crate::FieldWriter<'a, u32, DTUPD6_SPEC, u16, u16, 16, O>;
impl W {
    #[doc = "Bits 0:15 - Dead-Time Value Update for PWMHx Output"]
    #[inline(always)]
    pub fn dthupd(&mut self) -> DTHUPD_W<0> {
        DTHUPD_W::new(self)
    }
    #[doc = "Bits 16:31 - Dead-Time Value Update for PWMLx Output"]
    #[inline(always)]
    pub fn dtlupd(&mut self) -> DTLUPD_W<16> {
        DTLUPD_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Channel Dead Time Update Register (ch_num = 6)\n\nThis register you can [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dtupd6](index.html) module"]
pub struct DTUPD6_SPEC;
impl crate::RegisterSpec for DTUPD6_SPEC {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [dtupd6::W](W) writer structure"]
impl crate::Writable for DTUPD6_SPEC {
    type Writer = W;
}