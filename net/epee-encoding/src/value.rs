//! This module contains a [`EpeeValue`] trait and
//! impls for some possible base epee values.

use alloc::{string::String, vec, vec::Vec};
use core::{cmp::min, fmt::Debug};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use cuprate_fixed_bytes::{ByteArray, ByteArrayVec};
use cuprate_hex::{Hex, HexVec};

use crate::{
    io::{checked_read_primitive, checked_write_primitive},
    max_upfront_capacity,
    varint::{read_varint, write_varint},
    write_bytes, write_iterator, EpeeObject, Error, InnerMarker, Marker, Result,
    MAX_STRING_LEN_POSSIBLE,
};

/// A trait for epee values.
///
/// All [`EpeeObject`] objects automatically implement [`EpeeValue`].
pub trait EpeeValue: Sized {
    const MARKER: Marker;

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self>;

    fn should_write(&self) -> bool {
        true
    }

    /// This is different than default field values and instead is the default
    /// value of a whole type.
    ///
    /// For example a `Vec` has a default value of a zero length vec as when a
    /// sequence has no entries it is not encoded.
    fn epee_default_value() -> Option<Self> {
        None
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()>;
}

impl<T: EpeeObject> EpeeValue for T {
    const MARKER: Marker = Marker::new(InnerMarker::Object);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let mut skipped_objects = 0;
        crate::read_object(r, &mut skipped_objects)
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_varint(self.number_of_fields(), w)?;
        self.write_fields(w)
    }
}

impl<T: EpeeObject> EpeeValue for Vec<T> {
    const MARKER: Marker = T::MARKER.into_seq();

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if !marker.is_seq {
            return Err(Error::Format(
                "Marker is not sequence when a sequence was expected",
            ));
        }
        let len = read_varint(r)?;

        let individual_marker = Marker::new(marker.inner_marker);

        let mut res = Self::with_capacity(min(len, max_upfront_capacity::<T>()));
        for _ in 0..len {
            res.push(T::read(r, &individual_marker)?);
        }
        Ok(res)
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_iterator(self.into_iter(), w)
    }
}

impl<T: EpeeObject + Debug, const N: usize> EpeeValue for [T; N] {
    const MARKER: Marker = <T>::MARKER.into_seq();

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        let vec = Vec::<T>::read(r, marker)?;

        if vec.len() != N {
            return Err(Error::Format("Array has incorrect length"));
        }

        Ok(vec.try_into().unwrap())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_iterator(self.into_iter(), w)
    }
}

macro_rules! epee_numb {
    ($numb:ty, $marker:ident, $read_fn:ident, $write_fn:ident) => {
        impl EpeeValue for $numb {
            const MARKER: Marker = Marker::new(InnerMarker::$marker);

            fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
                if marker != &Self::MARKER {
                    return Err(Error::Format("Marker does not match expected Marker"));
                }

                checked_read_primitive(r, Buf::$read_fn)
            }

            fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
                checked_write_primitive(w, BufMut::$write_fn, self)
            }
        }
    };
}

epee_numb!(i64, I64, get_i64_le, put_i64_le);
epee_numb!(i32, I32, get_i32_le, put_i32_le);
epee_numb!(i16, I16, get_i16_le, put_i16_le);
epee_numb!(i8, I8, get_i8, put_i8);
epee_numb!(u8, U8, get_u8, put_u8);
epee_numb!(u16, U16, get_u16_le, put_u16_le);
epee_numb!(u32, U32, get_u32_le, put_u32_le);
epee_numb!(u64, U64, get_u64_le, put_u64_le);
epee_numb!(f64, F64, get_f64_le, put_f64_le);

impl EpeeValue for bool {
    const MARKER: Marker = Marker::new(InnerMarker::Bool);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        Ok(checked_read_primitive(r, Buf::get_u8)? != 0)
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        checked_write_primitive(w, BufMut::put_u8, if self { 1 } else { 0 })
    }
}

impl EpeeValue for Vec<u8> {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        if r.remaining() < len {
            return Err(Error::IO("Not enough bytes to fill object"));
        }

        let mut res = vec![0; len];
        r.copy_to_slice(&mut res);

        Ok(res)
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_bytes(self, w)
    }
}

impl EpeeValue for Bytes {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        if r.remaining() < len {
            return Err(Error::IO("Not enough bytes to fill object"));
        }

        Ok(r.copy_to_bytes(len))
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_bytes(self, w)
    }
}

impl EpeeValue for BytesMut {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        if r.remaining() < len {
            return Err(Error::IO("Not enough bytes to fill object"));
        }

        let mut bytes = Self::zeroed(len);
        r.copy_to_slice(&mut bytes);

        Ok(bytes)
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_bytes(self, w)
    }
}

impl<const N: usize> EpeeValue for ByteArrayVec<N> {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint::<_, usize>(r)?;
        if len > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::Format("Byte array exceeded max length"));
        }

        if r.remaining() < len {
            return Err(Error::IO("Not enough bytes to fill object"));
        }

        Self::try_from(r.copy_to_bytes(len)).map_err(|_| Error::Format("Field has invalid length"))
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::try_from(Bytes::new()).unwrap())
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        let bytes = self.take_bytes();
        write_bytes(bytes, w)
    }
}

impl<const N: usize> EpeeValue for ByteArray<N> {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if marker != &Self::MARKER {
            return Err(Error::Format("Marker does not match expected Marker"));
        }

        let len = read_varint::<_, usize>(r)?;
        if len != N {
            return Err(Error::Format("Byte array has incorrect length"));
        }

        if r.remaining() < N {
            return Err(Error::IO("Not enough bytes to fill object"));
        }

        Self::try_from(r.copy_to_bytes(N)).map_err(|_| Error::Format("Field has invalid length"))
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        let bytes = self.take_bytes();
        write_bytes(bytes, w)
    }
}

impl EpeeValue for String {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        let bytes = Vec::<u8>::read(r, marker)?;
        Self::from_utf8(bytes).map_err(|_| Error::Format("Invalid string"))
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_bytes(self, w)
    }
}

impl<const N: usize> EpeeValue for [u8; N] {
    const MARKER: Marker = Marker::new(InnerMarker::String);

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        let bytes = Vec::<u8>::read(r, marker)?;

        if bytes.len() != N {
            return Err(Error::Format("Byte array has incorrect length"));
        }

        Ok(bytes.try_into().unwrap())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_bytes(self, w)
    }
}

impl<const N: usize> EpeeValue for Vec<[u8; N]> {
    const MARKER: Marker = <[u8; N]>::MARKER.into_seq();

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        if !marker.is_seq {
            return Err(Error::Format(
                "Marker is not sequence when a sequence was expected",
            ));
        }

        let len = read_varint(r)?;

        let individual_marker = Marker::new(marker.inner_marker);

        let mut res = Self::with_capacity(min(len, max_upfront_capacity::<[u8; N]>()));
        for _ in 0..len {
            res.push(<[u8; N]>::read(r, &individual_marker)?);
        }
        Ok(res)
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_iterator(self.into_iter(), w)
    }
}

impl<const N: usize> EpeeValue for Hex<N> {
    const MARKER: Marker = <[u8; N] as EpeeValue>::MARKER;

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        Ok(Self(<[u8; N] as EpeeValue>::read(r, marker)?))
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        <[u8; N] as EpeeValue>::write(self.0, w)
    }
}

impl<const N: usize> EpeeValue for Vec<Hex<N>> {
    const MARKER: Marker = Vec::<[u8; N]>::MARKER;

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        Ok(Vec::<[u8; N]>::read(r, marker)?
            .into_iter()
            .map(Hex)
            .collect())
    }

    fn should_write(&self) -> bool {
        !self.is_empty()
    }

    fn epee_default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        write_iterator(self.into_iter(), w)
    }
}

impl EpeeValue for HexVec {
    const MARKER: Marker = <Vec<u8> as EpeeValue>::MARKER;

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        Ok(Self(<Vec<u8> as EpeeValue>::read(r, marker)?))
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        <Vec<u8> as EpeeValue>::write(self.0, w)
    }
}

macro_rules! epee_seq {
    ($val:ty) => {
        impl EpeeValue for Vec<$val> {
            const MARKER: Marker = <$val>::MARKER.into_seq();

            fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
                if !marker.is_seq {
                    return Err(Error::Format(
                        "Marker is not sequence when a sequence was expected",
                    ));
                }

                let len = read_varint(r)?;

                let individual_marker = Marker::new(marker.inner_marker.clone());

                let mut res = Vec::with_capacity(min(len, max_upfront_capacity::<$val>()));
                for _ in 0..len {
                    res.push(<$val>::read(r, &individual_marker)?);
                }
                Ok(res)
            }

            fn should_write(&self) -> bool {
                !self.is_empty()
            }

            fn epee_default_value() -> Option<Self> {
                Some(Vec::new())
            }

            fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
                write_iterator(self.into_iter(), w)
            }
        }

        impl<const N: usize> EpeeValue for [$val; N] {
            const MARKER: Marker = <$val>::MARKER.into_seq();

            fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
                let vec = Vec::<$val>::read(r, marker)?;

                if vec.len() != N {
                    return Err(Error::Format("Array has incorrect length"));
                }

                Ok(vec.try_into().unwrap())
            }

            fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
                write_iterator(self.into_iter(), w)
            }
        }
    };
}

epee_seq!(i64);
epee_seq!(i32);
epee_seq!(i16);
epee_seq!(i8);
epee_seq!(u64);
epee_seq!(u32);
epee_seq!(u16);
epee_seq!(f64);
epee_seq!(bool);
epee_seq!(Vec<u8>);
epee_seq!(HexVec);
epee_seq!(String);
epee_seq!(Bytes);
epee_seq!(BytesMut);

impl<T: EpeeValue> EpeeValue for Option<T> {
    const MARKER: Marker = T::MARKER;

    fn read<B: Buf>(r: &mut B, marker: &Marker) -> Result<Self> {
        Ok(Some(T::read(r, marker)?))
    }

    fn should_write(&self) -> bool {
        match self {
            Some(t) => t.should_write(),
            None => false,
        }
    }

    fn epee_default_value() -> Option<Self> {
        Some(None)
    }

    fn write<B: BufMut>(self, w: &mut B) -> Result<()> {
        match self {
            Some(t) => t.write(w)?,
            None => panic!("Can't write an Option::None value, this should be handled elsewhere"),
        }
        Ok(())
    }
}
