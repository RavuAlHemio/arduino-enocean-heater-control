#[doc = "Register `CSE` reader"]
pub struct R(crate::R<CSE_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<CSE_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<CSE_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<CSE_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `CSE` writer"]
pub struct W(crate::W<CSE_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<CSE_SPEC>;
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
impl From<crate::W<CSE_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<CSE_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `CSE` reader - Carrier Sense Errors"]
pub type CSE_R = crate::FieldReader<u8, u8>;
#[doc = "Field `CSE` writer - Carrier Sense Errors"]
pub type CSE_W<'a, const O: u8> = crate::FieldWriter<'a, u32, CSE_SPEC, u8, u8, 8, O>;
impl R {
    #[doc = "Bits 0:7 - Carrier Sense Errors"]
    #[inline(always)]
    pub fn cse(&self) -> CSE_R {
        CSE_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Carrier Sense Errors"]
    #[inline(always)]
    #[must_use]
    pub fn cse(&mut self) -> CSE_W<0> {
        CSE_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Carrier Sense Errors Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cse](index.html) module"]
pub struct CSE_SPEC;
impl crate::RegisterSpec for CSE_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [cse::R](R) reader structure"]
impl crate::Readable for CSE_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [cse::W](W) writer structure"]
impl crate::Writable for CSE_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets CSE to value 0"]
impl crate::Resettable for CSE_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
