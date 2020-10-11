//! Unit tests for the `slice` module.

#![cfg(test)]

use crate::prelude::*;

#[test]
fn construction() {
	#[cfg(not(miri))]
	use core::slice;

	let data = 0u8;
	let bits = data.view_bits::<LocalBits>();
	assert_eq!(bits.len(), 8);

	#[cfg(not(miri))]
	assert!(
		BitSlice::<LocalBits, u8>::from_slice(unsafe {
			slice::from_raw_parts(
				1usize as *const _,
				BitSlice::<LocalBits, u8>::MAX_ELTS,
			)
		})
		.is_none()
	);

	#[cfg(not(miri))]
	assert!(
		BitSlice::<LocalBits, u8>::from_slice_mut(unsafe {
			slice::from_raw_parts_mut(
				1usize as *mut _,
				BitSlice::<LocalBits, u8>::MAX_ELTS,
			)
		})
		.is_none()
	);

	assert_eq!(
		unsafe { crate::slice::bits_from_raw_parts(&data, 0, 8) },
		Some(bits)
	);
	assert!(
		unsafe {
			crate::slice::bits_from_raw_parts::<LocalBits, _>(&data, 0, !0)
		}
		.is_none()
	);

	let mut data = 0u8;
	assert_eq!(
		unsafe {
			crate::slice::bits_from_raw_parts_mut(&mut data as *mut _, 0, 8)
		},
		Some(data.view_bits_mut::<LocalBits>())
	);
}

#[test]
fn get_set() {
	let bits = bits![mut LocalBits, u8; 0; 8];

	for n in 0 .. 8 {
		assert!(!bits.get(n).unwrap());
		bits.set(n, true);
		assert!(bits.get(n).unwrap());
	}

	assert!(bits.get(9).is_none());
	assert!(bits.get_mut(9).is_none());
	assert!(bits.get(8 .. 10).is_none());
	assert!(bits.get_mut(8 .. 10).is_none());

	assert_eq!(bits.first(), Some(&true));
	*bits.first_mut().unwrap() = false;
	assert_eq!(bits.last(), Some(&true));
	*bits.last_mut().unwrap() = false;

	*crate::slice::BitSliceIndex::index_mut(1usize, bits) = false;
	assert_eq!(bits, bits![0, 0, 1, 1, 1, 1, 1, 0]);
	assert!(bits.get(100 ..).is_none());
	assert!(bits.get(.. 100).is_none());

	let (a, b) = (bits![mut Msb0, u8; 0, 1], bits![mut Lsb0, u16; 1, 0]);
	assert_eq!(a, bits![0, 1]);
	assert_eq!(b, bits![1, 0]);
	a.swap_with_bitslice(b);
	assert_eq!(a, bits![1, 0]);
	assert_eq!(b, bits![0, 1]);
}

#[test]
fn memcpy() {
	let mut dst = bitarr![0; 500];
	let src = bitarr![1; 500];

	//  Equal heads will fall into the fast path.
	dst[10 .. 20].copy_from_bitslice(&src[74 .. 84]);
	dst[100 .. 500].copy_from_bitslice(&src[36 .. 436]);

	//  Unequal heads will trip the slow path.
	dst[.. 490].copy_from_bitslice(&src[10 .. 500]);
}

#[test]
fn batch_copy() {
	let mut l = bitarr![Lsb0, usize; 0; 500];
	let mut m = bitarr![Msb0, usize; 0; 500];

	let l2 = bitarr![Lsb0, usize; 1; 500];
	let m2 = bitarr![Msb0, usize; 1; 500];

	assert!(l.not_any());
	l.copy_from_bitslice(&l2);
	assert!(l.all());

	assert!(m.not_any());
	m.copy_from_bitslice(&m2);
	assert!(m.all());
}

#[test]
fn query() {
	let data = [0x0Fu8, !0, 0xF0, 0, 0x0E];
	let bits = data.view_bits::<Msb0>();

	assert!(bits[36 .. 39].all());
	assert!(bits[4 .. 20].all());
	assert!(bits[.. 8].any());
	assert!(bits[4 .. 20].any());
	assert!(bits[32 ..].not_all());
	assert!(bits[.. 4].not_any());
	assert!(bits[.. 8].some());

	assert_eq!(bits[1 .. 7].count_ones(), 3);
	assert_eq!(bits[1 .. 7].count_zeros(), 3);
	assert_eq!(bits[.. 24].count_ones(), 16);
	assert_eq!(bits[16 ..].count_zeros(), 17);

	assert!(!bits![0].contains(bits![0, 1]));
	assert!(bits![0, 1, 0].contains(bits![1, 0]));
	assert!(bits![0, 1, 0].starts_with(bits![0, 1]));
	assert!(bits![0, 1, 0].ends_with(bits![1, 0]));
}

#[test]
fn modify() {
	let mut data = 0b0000_1111u8;

	let bits = data.view_bits_mut::<LocalBits>();
	bits.swap(3, 4);
	assert_eq!(data, 0b0001_0111);

	let bits = data.view_bits_mut::<Lsb0>();
	bits[1 .. 7].reverse();
	assert_eq!(data, 0b0110_1001);
	data.view_bits_mut::<Msb0>()[1 .. 7].reverse();

	let bits = data.view_bits_mut::<Msb0>();
	bits.copy_within(2 .. 4, 0);
	assert_eq!(data, 0b0101_0111);

	let bits = data.view_bits_mut::<Msb0>();
	bits.copy_within(5 .., 2);
	assert_eq!(data, 0b0111_1111);
}

#[test]
fn split() {
	assert!(
		BitSlice::<LocalBits, usize>::empty()
			.split_first()
			.is_none()
	);
	assert_eq!(
		1u8.view_bits::<Lsb0>().split_first(),
		Some((&true, bits![Lsb0, u8; 0; 7]))
	);

	assert!(
		BitSlice::<LocalBits, usize>::empty_mut()
			.split_first_mut()
			.is_none()
	);
	let mut data = 0u8;
	let (head, _) = data.view_bits_mut::<Lsb0>().split_first_mut().unwrap();
	head.set(true);
	assert_eq!(data, 1);

	assert!(BitSlice::<LocalBits, usize>::empty().split_last().is_none());
	assert_eq!(
		1u8.view_bits::<Msb0>().split_last(),
		Some((&true, bits![Msb0, u8; 0; 7]))
	);

	assert!(
		BitSlice::<LocalBits, usize>::empty_mut()
			.split_first_mut()
			.is_none()
	);
	let mut data = 0u8;
	let (head, _) = data.view_bits_mut::<Msb0>().split_last_mut().unwrap();
	head.set(true);
	assert_eq!(data, 1);

	let mut data = 0b0000_1111u8;

	let bits = data.view_bits::<Msb0>();
	let (left, right) = bits.split_at(4);
	assert!(left.not_any());
	assert!(right.all());

	let bits = data.view_bits_mut::<Msb0>();
	let (left, right) = bits.split_at_mut(4);
	left.set_all(true);
	right.set_all(false);
	assert_eq!(data, 0b1111_0000u8);
}

#[test]
fn iterators() {
	assert!(bits![0; 2].iter().nth(2).is_none());
	assert!(bits![0; 2].iter().nth_back(2).is_none());

	let bits = bits![mut 0; 4];

	assert!(bits.chunks(2).nth(2).is_none());
	assert!(bits.chunks(2).nth_back(2).is_none());
	assert!(bits.chunks_mut(2).nth(2).is_none());
	assert!(bits.chunks_mut(2).nth_back(2).is_none());

	assert!(bits.rchunks(2).nth(2).is_none());
	assert!(bits.rchunks(2).nth_back(2).is_none());
	assert!(bits.rchunks_mut(2).nth(2).is_none());
	assert!(bits.rchunks_mut(2).nth_back(2).is_none());
	assert!(bits![mut].rchunks_mut(1).next().is_none());

	bits![Msb0, u8; 0, 1, 0, 0, 1, 0, 0, 0]
		.split(|_, bit| *bit)
		.zip([1usize, 2, 3].iter())
		.for_each(|(bits, len)| assert_eq!(bits.len(), *len));

	let mut data = 0b0100_1000u8;
	data.view_bits_mut::<Msb0>()
		.split_mut(|_, bit| *bit)
		.zip([1usize, 2, 3].iter())
		.for_each(|(bits, len)| {
			assert_eq!(bits.len(), *len);
			bits.set_all(true)
		});
	assert_eq!(data, !0);

	bits![Msb0, u8; 0, 1, 0, 0, 1, 0, 0, 0]
		.rsplit(|_, bit| *bit)
		.zip([3usize, 2, 1].iter())
		.for_each(|(bits, len)| assert_eq!(bits.len(), *len));

	let mut data = 0b0100_1000u8;
	data.view_bits_mut::<Msb0>()
		.rsplit_mut(|_, bit| *bit)
		.zip([3usize, 2, 1].iter())
		.for_each(|(bits, len)| {
			assert_eq!(bits.len(), *len);
			bits.set_all(true)
		});
	assert_eq!(data, !0);

	bits![Msb0, u8; 0, 1, 0, 0, 1, 0, 0, 0]
		.splitn(2, |_, bit| *bit)
		.zip([1usize, 6].iter())
		.for_each(|(bits, len)| assert_eq!(bits.len(), *len));

	let mut data = 0b0100_1000u8;
	data.view_bits_mut::<Msb0>()
		.splitn_mut(2, |_, bit| *bit)
		.zip([1usize, 6].iter())
		.for_each(|(bits, len)| {
			assert_eq!(bits.len(), *len);
			bits.set_all(true)
		});
	assert_eq!(data, !0);

	bits![Msb0, u8; 0, 1, 0, 0, 1, 0, 0, 0]
		.rsplitn(2, |_, bit| *bit)
		.zip([3usize, 4].iter())
		.for_each(|(bits, len)| assert_eq!(bits.len(), *len));

	let mut data = 0b0100_1000u8;
	data.view_bits_mut::<Msb0>()
		.rsplitn_mut(2, |_, bit| *bit)
		.zip([3usize, 4].iter())
		.for_each(|(bits, len)| {
			assert_eq!(bits.len(), *len);
			bits.set_all(true)
		});
	assert_eq!(data, !0);
}

#[test]
fn alignment() {
	let mut data = [0u16; 5];
	let addr = &data as *const [u16; 5] as *const u16 as usize;
	let bits = data.view_bits_mut::<LocalBits>();

	let (head, body, tail) = unsafe { bits[5 .. 75].align_to_mut::<u32>() };

	//  `data` is aligned to the back half of a `u32`
	if addr % 4 == 2 {
		assert_eq!(head.len(), 11);
		assert_eq!(body.len(), 59);
		assert!(tail.is_empty());
	}
	//  `data` is aligned to the front half of a `u32`
	else {
		assert!(head.is_empty());
		assert_eq!(body.len(), 64);
		assert_eq!(tail.len(), 6);
	}
}

#[test]
#[cfg(feature = "alloc")]
fn repetition() {
	let bits = bits![0, 0, 1, 1];
	let bv = bits.repeat(2);
	assert_eq!(bv, bits![0, 0, 1, 1, 0, 0, 1, 1]);
}

#[test]
fn pointer_offset() {
	let data = [0u16; 2];
	let bits = data.view_bits::<Msb0>();

	let a = &bits[10 .. 11];
	let b = &bits[20 .. 21];
	assert_eq!(a.offset_from(b), 10);
}

#[test]
fn shift() {
	let bits = bits![mut 1; 6];
	bits.shift_left(0);
	bits.shift_right(0);
	assert_eq!(bits, bits![1; 6]);

	bits.shift_left(4);
	assert_eq!(bits, bits![1, 1, 0, 0, 0, 0]);
	bits.shift_right(2);
	assert_eq!(bits, bits![0, 0, 1, 1, 0, 0]);
}

#[test]
fn invert() {
	let mut data = [0u8; 4];
	let bits = data.view_bits_mut::<Lsb0>();

	let inv = !&mut bits[2 .. 6];
	assert!(inv.all());

	let inv = !&mut bits[12 .. 28];
	assert!(inv.all());

	assert_eq!(data, [0x3C, 0xF0, 0xFF, 0x0F]);
}

#[test]
fn rotate() {
	let bits = bits![mut 0, 1, 0, 0, 1, 0];

	bits.rotate_left(0);
	bits.rotate_right(0);
	bits.rotate_left(6);
	bits.rotate_right(6);

	assert_eq!(bits, bits![0, 1, 0, 0, 1, 0]);
}
