#[doc = "Register `DT4` reader"]
pub struct R(crate::R<DT4_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<DT4_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<DT4_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<DT4_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `DT4` writer"]
pub struct W(crate::W<DT4_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<DT4_SPEC>;
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
impl From<crate::W<DT4_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<DT4_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `DTH` reader - Dead-Time Value for PWMHx Output"]
pub type DTH_R = crate::FieldReader<u16, u16>;
#[doc = "Field `DTH` writer - Dead-Time Value for PWMHx Output"]
pub type DTH_W<'a, const O: u8> = crate::FieldWriter<'a, u32, DT4_SPEC, u16, u16, 16, O>;
#[doc = "Field `DTL` reader - Dead-Time Value for PWMLx Output"]
pub type DTL_R = crate::FieldReader<u16, u16>;
#[doc = "Field `DTL` writer - Dead-Time Value for PWMLx Output"]
pub type DTL_W<'a, const O: u8> = crate::FieldWriter<'a, u32, DT4_SPEC, u16, u16, 16, O>;
impl R {
    #[doc = "Bits 0:15 - Dead-Time Value for PWMHx Output"]
    #[inline(always)]
    pub fn dth(&self) -> DTH_R {
        DTH_R::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Dead-Time Value for PWMLx Output"]
    #[inline(always)]
    pub fn dtl(&self) -> DTL_R {
        DTL_R::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Dead-Time Value for PWMHx Output"]
    #[inline(always)]
    #[must_use]
    pub fn dth(&mut self) -> DTH_W<0> {
        DTH_W::new(self)
    }
    #[doc = "Bits 16:31 - Dead-Time Value for PWMLx Output"]
    #[inline(always)]
    #[must_use]
    pub fn dtl(&mut self) -> DTL_W<16> {
        DTL_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Channel Dead Time Register (ch_num = 4)\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dt4](index.html) module"]
pub struct DT4_SPEC;
impl crate::RegisterSpec for DT4_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [dt4::R](R) reader structure"]
impl crate::Readable for DT4_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [dt4::W](W) writer structure"]
impl crate::Writable for DT4_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets DT4 to value 0"]
impl crate::Resettable for DT4_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
