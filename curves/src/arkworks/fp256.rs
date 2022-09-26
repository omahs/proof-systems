//! While we could have create a custom type Fp256, and type aliases for Fp and Fq,
//! We create two custom types using this macro: one for Fp and one for Fq.
//! This makes bindings easier to reason about.
//!
//! The strategy used is to create wrappers around `ark_ff::Fp256<Fp_params>` and `ark_ff::Fp256<Fq_params>`,
//! and implement `ark_ff::Field` and related traits that are needed
//! to pretend that these are the actual types.
//!
//! Note: We can't use ark_ff::Fp256 directly because it doesn't implement `ocaml::ToValue`.
//! And we can't implement `ocaml::ToValue` for `ark_ff::Fp256` because it's not defined in this crate.

macro_rules! impl_field {
    ($name: ident, $CamlF: ident, $ArkF: ty, $Params: ty) => {
        paste! {
            //
            // Conversions
            //

            impl From<$ArkF> for $CamlF {
                fn from(ark_fp: $ArkF) -> Self {
                    Self(ark_fp)
                }
            }

            impl From<&$ArkF> for $CamlF {
                fn from(ark_fp: &$ArkF) -> Self {
                    Self(*ark_fp)
                }
            }

            impl From<$CamlF> for $ArkF {
                fn from(fp: $CamlF) -> Self {
                    fp.0
                }
            }

            impl From<&$CamlF> for $ArkF {
                fn from(fp: &$CamlF) -> Self {
                    fp.0
                }
            }

            //
            //
            //

            impl Default for $CamlF {
                fn default() -> Self {
                    ark_ff::Fp256::default().into()
                }
            }
            impl Hash for $CamlF {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.0.hash(state);
                }
            }
            impl Clone for $CamlF {
                fn clone(&self) -> Self {
                    self.0.clone().into()
                }
            }
            impl Copy for $CamlF {}
            impl Debug for $CamlF {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    f.debug_tuple("Fp256").field(&self.0).finish()
                }
            }
            impl PartialEq for $CamlF {
                fn eq(&self, other: &Self) -> bool {
                    self.0.eq(&other.0)
                }
            }
            impl Eq for $CamlF {}

            //
            //
            //

            impl $CamlF {
                pub fn new(x: ark_ff::BigInteger256) -> Self {
                    ark_ff::Fp256::new(x).into()
                }
            }

            impl ark_ff::Zero for $CamlF {
                #[inline]
                fn zero() -> Self {
                    ark_ff::Fp256::zero().into()
                }

                #[inline]
                fn is_zero(&self) -> bool {
                    self.0.is_zero()
                }
            }

            impl ark_ff::One for $CamlF {
                #[inline]
                fn one() -> Self {
                    ark_ff::Fp256::one().into()
                }

                #[inline]
                fn is_one(&self) -> bool {
                    self.0.is_one()
                }
            }

            impl ark_ff::Field for $CamlF {
                type BasePrimeField = Self;

                fn extension_degree() -> u64 {
                    $ArkF::extension_degree()
                }

                fn from_base_prime_field_elems(elems: &[Self::BasePrimeField]) -> Option<Self> {
                    // TODO: this looks suboptimal
                    let elems: Vec<<$ArkF as ark_ff::Field>::BasePrimeField> =
                        elems.iter().map(|x| x.0).collect();
                    ark_ff::Fp256::from_base_prime_field_elems(&elems).map(Into::into)
                }

                #[inline]
                fn double(&self) -> Self {
                    self.0.double().into()
                }

                #[inline]
                fn double_in_place(&mut self) -> &mut Self {
                    self.0.double_in_place();
                    self
                }

                #[inline]
                fn characteristic() -> &'static [u64] {
                    $ArkF::characteristic()
                }

                #[inline]
                fn from_random_bytes_with_flags<F>(bytes: &[u8]) -> Option<(Self, F)>
                where
                    F: ark_serialize::Flags,
                {
                    ark_ff::Fp256::from_random_bytes_with_flags(bytes).map(|(x, f)| (x.into(), f))
                }

                #[inline]
                fn square(&self) -> Self {
                    self.0.square().into()
                }

                #[inline]
                fn square_in_place(&mut self) -> &mut Self {
                    self.0.square_in_place();
                    self
                }

                #[inline]
                fn inverse(&self) -> Option<Self> {
                    self.0.inverse().map(Into::into)
                }

                fn inverse_in_place(&mut self) -> Option<&mut Self> {
                    if self.0.inverse_in_place().is_some() {
                        Some(self)
                    } else {
                        None
                    }
                }

                /// The Frobenius map has no effect in a prime field.
                #[inline]
                fn frobenius_map(&mut self, x: usize) {
                    self.0.frobenius_map(x)
                }
            }

            impl ark_ff::PrimeField for $CamlF {
                type Params = $Params;
                type BigInt = ark_ff::BigInteger256;

                #[inline]
                fn from_repr(r: Self::BigInt) -> Option<Self> {
                    ark_ff::Fp256::from_repr(r).map(Into::into)
                }

                fn into_repr(&self) -> Self::BigInt {
                    self.0.into_repr()
                }
            }

            impl ark_ff::FftField for $CamlF {
                type FftParams = $Params;

                #[inline]
                fn two_adic_root_of_unity() -> Self {
                    ark_ff::Fp256::two_adic_root_of_unity().into()
                }

                #[inline]
                fn large_subgroup_root_of_unity() -> Option<Self> {
                    ark_ff::Fp256::large_subgroup_root_of_unity().map(Into::into)
                }

                #[inline]
                fn multiplicative_generator() -> Self {
                    ark_ff::Fp256::multiplicative_generator().into()
                }
            }

            impl ark_ff::SquareRootField for $CamlF {
                #[inline]
                fn legendre(&self) -> ark_ff::LegendreSymbol {
                    self.0.legendre()
                }

                #[inline]
                fn sqrt(&self) -> Option<Self> {
                    self.0.sqrt().map(Into::into)
                }

                fn sqrt_in_place(&mut self) -> Option<&mut Self> {
                    if self.0.sqrt_in_place().is_some() {
                        Some(self)
                    } else {
                        None
                    }
                }
            }

            impl Ord for $CamlF {
                #[inline(always)]
                fn cmp(&self, other: &Self) -> Ordering {
                    self.0.cmp(&other.0)
                }
            }

            impl PartialOrd for $CamlF {
                #[inline(always)]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    self.0.partial_cmp(&other.0)
                }
            }

            impl From<u128> for $CamlF {
                fn from(other: u128) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<i128> for $CamlF {
                fn from(other: i128) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<bool> for $CamlF {
                fn from(other: bool) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<u64> for $CamlF {
                fn from(other: u64) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<i64> for $CamlF {
                fn from(other: i64) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<u32> for $CamlF {
                fn from(other: u32) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<i32> for $CamlF {
                fn from(other: i32) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<u16> for $CamlF {
                fn from(other: u16) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<i16> for $CamlF {
                fn from(other: i16) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<u8> for $CamlF {
                fn from(other: u8) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl From<i8> for $CamlF {
                fn from(other: i8) -> Self {
                    ark_ff::Fp256::from(other).into()
                }
            }

            impl ark_ff::ToBytes for $CamlF {
                #[inline]
                fn write<W: Write>(&self, writer: W) -> IoResult<()> {
                    self.0.write(writer)
                }
            }

            impl ark_ff::FromBytes for $CamlF {
                #[inline]
                fn read<R: Read>(reader: R) -> IoResult<Self> {
                    ark_ff::Fp256::read(reader).map(Into::into)
                }
            }

            impl FromStr for $CamlF {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    ark_ff::Fp256::from_str(s).map(Into::into)
                }
            }

            impl Display for $CamlF {
                #[inline]
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    Display::fmt(&self.0, f)
                }
            }

            impl Neg for $CamlF {
                type Output = Self;
                #[inline]
                #[must_use]
                fn neg(self) -> Self {
                    self.0.neg().into()
                }
            }

            impl<'a> Add<&'a $CamlF> for $CamlF {
                type Output = Self;

                #[inline]
                fn add(self, other: &Self) -> Self {
                    self.0.add(&other.0).into()
                }
            }

            impl<'a> Sub<&'a $CamlF> for $CamlF {
                type Output = Self;

                #[inline]
                fn sub(self, other: &Self) -> Self {
                    self.0.sub(&other.0).into()
                }
            }

            impl<'a> Mul<&'a $CamlF> for $CamlF {
                type Output = Self;

                #[inline]
                fn mul(self, other: &Self) -> Self {
                    self.0.mul(&other.0).into()
                }
            }

            impl<'a> Div<&'a $CamlF> for $CamlF {
                type Output = Self;

                /// Returns `self * other.inverse()` if `other.inverse()` is `Some`, and
                /// panics otherwise.
                #[inline]
                fn div(self, other: &Self) -> Self {
                    self.0.div(&other.0).into()
                }
            }

            impl<'a> AddAssign<&'a Self> for $CamlF {
                #[inline]
                fn add_assign(&mut self, other: &Self) {
                    self.0.add_assign(&other.0);
                }
            }

            impl<'a> SubAssign<&'a Self> for $CamlF {
                #[inline]
                fn sub_assign(&mut self, other: &Self) {
                    self.0.sub_assign(&other.0);
                }
            }

            impl<'a> MulAssign<&'a Self> for $CamlF {
                #[inline]
                fn mul_assign(&mut self, other: &Self) {
                    self.0.mul_assign(&other.0);
                }
            }

            impl<'a> DivAssign<&'a Self> for $CamlF {
                #[inline]
                fn div_assign(&mut self, other: &Self) {
                    self.0.div_assign(&other.0);
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::Add<Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn add(self, other: Self) -> Self {
                    self.0.add(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::Add<&'a mut Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn add(self, other: &'a mut Self) -> Self {
                    self.0.add(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::Sub<Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn sub(self, other: Self) -> Self {
                    self.0.sub(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::Sub<&'a mut Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn sub(self, other: &'a mut Self) -> Self {
                    self.0.sub(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::iter::Sum<Self> for $CamlF {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    $ArkF::sum(iter.map(|x| x.0)).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::iter::Sum<&'a Self> for $CamlF {
                fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                    $ArkF::sum(iter.map(|x| x.0)).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::AddAssign<Self> for $CamlF {
                fn add_assign(&mut self, other: Self) {
                    self.0.add_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::SubAssign<Self> for $CamlF {
                fn sub_assign(&mut self, other: Self) {
                    self.0.sub_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::AddAssign<&'a mut Self> for $CamlF {
                fn add_assign(&mut self, other: &'a mut Self) {
                    self.0.add_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::SubAssign<&'a mut Self> for $CamlF {
                fn sub_assign(&mut self, other: &'a mut Self) {
                    self.0.sub_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::Mul<Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn mul(self, other: Self) -> Self {
                    self.0.mul(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::Div<Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn div(self, other: Self) -> Self {
                    self.0.div(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::Mul<&'a mut Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn mul(self, other: &'a mut Self) -> Self {
                    self.0.mul(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::Div<&'a mut Self> for $CamlF {
                type Output = Self;

                #[inline]
                fn div(self, other: &'a mut Self) -> Self {
                    self.0.div(other.0).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::iter::Product<Self> for $CamlF {
                fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                    ark_ff::Fp256::product(iter.map(|x| x.0)).into()
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::iter::Product<&'a Self> for $CamlF {
                fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                    ark_ff::Fp256::product(iter.map(|x| x.0)).into()
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::MulAssign<Self> for $CamlF {
                fn mul_assign(&mut self, other: Self) {
                    self.0.mul_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::DivAssign<&'a mut Self> for $CamlF {
                fn div_assign(&mut self, other: &'a mut Self) {
                    self.0.div_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl<'a> core::ops::MulAssign<&'a mut Self> for $CamlF {
                fn mul_assign(&mut self, other: &'a mut Self) {
                    self.0.mul_assign(&other.0)
                }
            }

            #[allow(unused_qualifications)]
            impl core::ops::DivAssign<Self> for $CamlF {
                fn div_assign(&mut self, other: Self) {
                    self.0.div_assign(&other.0)
                }
            }

            impl zeroize::Zeroize for $CamlF {
                // The phantom data does not contain element-specific data
                // and thus does not need to be zeroized.
                fn zeroize(&mut self) {
                    self.0.zeroize();
                }
            }

            impl Into<ark_ff::BigInteger256> for $CamlF {
                fn into(self) -> ark_ff::BigInteger256 {
                    self.0.into()
                }
            }

            impl From<ark_ff::BigInteger256> for $CamlF {
                /// Converts `Self::BigInteger` into `Self`
                ///
                /// # Panics
                /// This method panics if `int` is larger than `P::MODULUS`.
                fn from(int: ark_ff::BigInteger256) -> Self {
                    ark_ff::Fp256::from(int).into()
                }
            }

            impl From<num_bigint::BigUint> for $CamlF {
                #[inline]
                fn from(val: num_bigint::BigUint) -> Self {
                    ark_ff::Fp256::from(val).into()
                }
            }

            impl Into<num_bigint::BigUint> for $CamlF {
                #[inline]
                fn into(self) -> num_bigint::BigUint {
                    self.0.into()
                }
            }

            impl ark_serialize::CanonicalSerializeWithFlags for $CamlF {
                fn serialize_with_flags<W: std::io::Write, F: ark_serialize::Flags>(
                    &self,
                    writer: W,
                    flags: F,
                ) -> Result<(), ark_serialize::SerializationError> {
                    self.0.serialize_with_flags(writer, flags)
                }

                fn serialized_size_with_flags<F: ark_serialize::Flags>(&self) -> usize {
                    self.0.serialized_size_with_flags::<F>()
                }
            }

            impl ark_serialize::CanonicalSerialize for $CamlF {
                #[inline]
                fn serialize<W: std::io::Write>(
                    &self,
                    writer: W,
                ) -> Result<(), ark_serialize::SerializationError> {
                    self.0.serialize(writer)
                }

                #[inline]
                fn serialized_size(&self) -> usize {
                    self.0.serialized_size()
                }
            }

            impl ark_serialize::CanonicalDeserializeWithFlags for $CamlF {
                fn deserialize_with_flags<R: std::io::Read, F: ark_serialize::Flags>(
                    reader: R,
                ) -> Result<(Self, F), ark_serialize::SerializationError> {
                    ark_ff::Fp256::deserialize_with_flags::<R, F>(reader).map(|(x, f)| (x.into(), f))
                }
            }

            impl ark_serialize::CanonicalDeserialize for $CamlF {
                fn deserialize<R: std::io::Read>(reader: R) -> Result<Self, ark_serialize::SerializationError> {
                    ark_ff::Fp256::deserialize(reader).map(Into::into)
                }
            }

            impl ark_std::rand::distributions::Distribution<$CamlF>
                for ark_std::rand::distributions::Standard
            {
                #[inline]
                fn sample<R: ark_std::rand::Rng + ?Sized>(&self, rng: &mut R) -> $CamlF {
                    let ark_fp: $ArkF = self.sample(rng);
                    $CamlF(ark_fp)
                }
            }

        }
    }
}

// Rust note: this is how we can use this macro within the crate.
pub(crate) use impl_field;
