#[doc = "Register `CPRD4` reader"]
pub struct R(crate::R<CPRD4_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<CPRD4_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<CPRD4_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<CPRD4_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `CPRD4` writer"]
pub struct W(crate::W<CPRD4_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<CPRD4_SPEC>;
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
impl From<crate::W<CPRD4_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<CPRD4_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `CPRD` reader - Channel Period"]
pub type CPRD_R = crate::FieldReader<u32, u32>;
#[doc = "Field `CPRD` writer - Channel Period"]
pub type CPRD_W<'a, const O: u8> = crate::FieldWriter<'a, u32, CPRD4_SPEC, u32, u32, 24, O>;
impl R {
    #[doc = "Bits 0:23 - Channel Period"]
    #[inline(always)]
    pub fn cprd(&self) -> CPRD_R {
        CPRD_R::new((self.bits & 0x00ff_ffff) as u32)
    }
}
impl W {
    #[doc = "Bits 0:23 - Channel Period"]
    #[inline(always)]
    pub fn cprd(&mut self) -> CPRD_W<0> {
        CPRD_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Channel Period Register (ch_num = 4)\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cprd4](index.html) module"]
pub struct CPRD4_SPEC;
impl crate::RegisterSpec for CPRD4_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [cprd4::R](R) reader structure"]
impl crate::Readable for CPRD4_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [cprd4::W](W) writer structure"]
impl crate::Writable for CPRD4_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets CPRD4 to value 0"]
impl crate::Resettable for CPRD4_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
