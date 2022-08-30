#[doc = "Register `CDTYUPD7` writer"]
pub struct W(crate::W<CDTYUPD7_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<CDTYUPD7_SPEC>;
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
impl From<crate::W<CDTYUPD7_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<CDTYUPD7_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `CDTYUPD` writer - Channel Duty-Cycle Update"]
pub type CDTYUPD_W<'a, const O: u8> = crate::FieldWriter<'a, u32, CDTYUPD7_SPEC, u32, u32, 24, O>;
impl W {
    #[doc = "Bits 0:23 - Channel Duty-Cycle Update"]
    #[inline(always)]
    pub fn cdtyupd(&mut self) -> CDTYUPD_W<0> {
        CDTYUPD_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Channel Duty Cycle Update Register (ch_num = 7)\n\nThis register you can [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cdtyupd7](index.html) module"]
pub struct CDTYUPD7_SPEC;
impl crate::RegisterSpec for CDTYUPD7_SPEC {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [cdtyupd7::W](W) writer structure"]
impl crate::Writable for CDTYUPD7_SPEC {
    type Writer = W;
}
