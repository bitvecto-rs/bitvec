/*! Controls for accessing memory.

[`bitvec`] allows a program to produce handles over memory that do not logically
alias their bits, but may alias in hardware. This module provides a unified
interface for memory accesses that can be specialized to handle aliased and
unaliased access events.

The [`BitAccess`] trait provides capabilities to access bits in memory elements
through shared references, and its implementations are responsible for
coördinating synchronization and contention as needed.

[`BitAccess`]: self::BitAccess
[`bitvec`]: crate
!*/

use crate::{
	index::{
		BitIdx,
		BitMask,
	},
	mem::BitRegister,
	order::BitOrder,
};

use core::{
	cell::Cell,
	fmt::Debug,
	sync::atomic::Ordering,
};

use radium::Radium;

#[cfg(feature = "atomic")]
use core::sync::atomic::{
	AtomicU16,
	AtomicU32,
	AtomicU8,
	AtomicUsize,
};

#[cfg(all(feature = "atomic", target_pointer_width = "64"))]
use core::sync::atomic::AtomicU64;

/** Selects instructions used when accessing a memory location.

This trait abstracts over the specific instructions used to perform accesses to
memory locations, so that use sites elsewhere in the crate can select their
required behavior without changing the interface.

This is automatically implemented for all types that permit shared/mutable
memory access to register types through the [`radium`] crate. Its use is
constrained in the [`store`] module.

This trait is only ever used by [`bitvec`] internals, and is never exposed outside
the crate. It must be `pub`lic so that it can be used as an associated item in
[`BitStore`], even though it is never made accessible.

[`BitStore`]: crate::store::BitStore
[`bitvec`]: crate
[`radium`]: radium
[`store`]: crate::store
**/
pub trait BitAccess: Debug + Radium + Sized
where <Self as Radium>::Item: BitRegister
{
	/// Sets one bit in a memory element to `0`.
	///
	/// # Type Parameters
	///
	/// - `O`: A bit ordering.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `index`: The semantic index of the bit in `*self` to be erased.
	///
	/// # Effects
	///
	/// The memory element at `*self` has the bit corresponding to `index` set
	/// to `0`, and all other bits are unchanged.
	#[inline]
	fn clear_bit<O>(&self, index: BitIdx<<Self as Radium>::Item>)
	where O: BitOrder {
		self.fetch_and(!index.select::<O>().value(), Ordering::Relaxed);
	}

	/// Set any number of bits in a memory element to `0`.
	///
	/// The mask provided to this method must be constructed from indices that
	/// are valid in the caller’s context. As the mask is already computed by
	/// the caller, this does not take an ordering type parameter.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `mask`: A mask of any number of bits. This is a selection mask of bits
	///   to modify.
	///
	/// # Effects
	///
	/// All bits in `*self` that are selected (set to `1`) in `mask` will be set
	/// to `0`, and all bits in `*self` that are not selected (set to `0`) in
	/// `mask` will be unchanged.
	///
	/// Do not invert the mask prior to calling this function in order to save
	/// the unselected bits and erase the selected bits. `BitMask` is a
	/// selection type, not a bitwise-operation argument.
	#[inline]
	fn clear_bits(&self, mask: BitMask<<Self as Radium>::Item>) {
		self.fetch_and(!mask.value(), Ordering::Relaxed);
	}

	/// Sets one bit in a memory element to `1`.
	///
	/// # Type Parameters
	///
	/// - `O`: A bit ordering.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `index`: The semantic index of the bit in `*self` to be written.
	///
	/// # Effects
	///
	/// The memory element at `*self` has the bit corresponding to `index` set
	/// to `1`, and all other bits are unchanged.
	#[inline]
	fn set_bit<O>(&self, index: BitIdx<<Self as Radium>::Item>)
	where O: BitOrder {
		self.fetch_or(index.select::<O>().value(), Ordering::Relaxed);
	}

	/// Sets any number of bits in a memory element to `1`.
	///
	/// The mask provided to this method must be constructed from indices that
	/// are valid in the caller’s context. As the mask is already computed by
	/// the caller, this does not need to take an ordering type parameter.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `mask`: A mask of any number of bits. This is a selection mask of bits
	///   to modify.
	///
	/// # Effects
	///
	/// All bits in `*self` that are selected (set to `1`) in `mask` will be set
	/// to `1`, and all bits in `*self` that are not selected (set to `0`) in
	/// `mask` will be unchanged.
	#[inline]
	fn set_bits(&self, mask: BitMask<<Self as Radium>::Item>) {
		self.fetch_or(mask.value(), Ordering::Relaxed);
	}

	/// Inverts the value of one bit in a memory element.
	///
	/// # Type Parameters
	///
	/// - `O`: A bit ordering.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `index`: The semantic index of the bit in `*self` to be inverted.
	///
	/// # Effects
	///
	/// The memory element at `*self` has the bit corresponding to `index` set
	/// to the opposite of its current value. All other bits are unchanged.
	#[inline]
	fn invert_bit<O>(&self, index: BitIdx<<Self as Radium>::Item>)
	where O: BitOrder {
		self.fetch_xor(index.select::<O>().value(), Ordering::Relaxed);
	}

	/// Invert any number of bits in a memory element.
	///
	/// The mask provided to this method must be constructed from indices that
	/// are valid in the caller’s context. As the mask is already computed by
	/// the caller, this does not take an ordering type parameter.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `mask`: A mask of any number of bits. This is a selection mask of bits
	///   to modify.
	///
	/// # Effects
	///
	/// All bits in `*self` that are selected (set to `1`) in `mask` will be set
	/// to the opposite of their current value, and all bits in `*self` that are
	/// not selected (set to `0`) in `mask` will be unchanged.
	#[inline]
	fn invert_bits(&self, mask: BitMask<<Self as Radium>::Item>) {
		self.fetch_xor(mask.value(), Ordering::Relaxed);
	}

	/// Writes a bit to an index within the `self` element.
	///
	/// # Type Parameters
	///
	/// - `O`: A bit ordering.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `index`: The semantic index of the bit in `*self` to be written.
	/// - `value`: The bit value to write into `*self` at `index`.
	///
	/// # Effects
	///
	/// The bit in `*self` at `index` is set to the `value` bit.
	#[inline]
	fn write_bit<O>(&self, index: BitIdx<<Self as Radium>::Item>, value: bool)
	where O: BitOrder {
		if value {
			self.set_bit::<O>(index);
		}
		else {
			self.clear_bit::<O>(index);
		}
	}

	/// Writes a bit value to any number of bits within the `self` element.
	///
	/// The mask provided to this method must be constructed from indices that
	/// are valid in the caller’s context. As the mask is already computed by
	/// the caller, this does not need to take an ordering type parameter.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `mask`: A mask of any number of bits. This is a selection mask of bits
	///   to modify.
	///
	/// # Effects
	///
	/// All bits in `*self` that are selected (set to `1`) in `mask` will be set
	/// to `value`, and all bits in `*self` that are not selected (set to `0`)
	/// in `mask` will be unchanged.
	#[inline]
	fn write_bits(&self, mask: BitMask<<Self as Radium>::Item>, value: bool) {
		if value {
			self.set_bits(mask);
		}
		else {
			self.clear_bits(mask);
		}
	}

	/// Gets the function that writes `value` to an index.
	///
	/// # Type Parameters
	///
	/// - `O`: A bit ordering.
	///
	/// # Parameters
	///
	/// - `value`: The bit that will be directly written by the returned
	///   function.
	///
	/// # Returns
	///
	/// A function which, when applied to a reference and an index, will write
	/// `value` into memory. If `value` is `false`, then this produces
	/// [`clear_bit`]; if it is `true`, then this produces [`set_bit`].
	///
	/// [`clear_bit`]: Self::clear_bit
	/// [`set_bit`]: Self::set_bit
	#[inline]
	fn get_writer<O>(
		value: bool,
	) -> for<'a> fn(&'a Self, BitIdx<<Self as Radium>::Item>)
	where O: BitOrder {
		[Self::clear_bit::<O>, Self::set_bit::<O>][value as usize]
	}

	/// Gets the function that writes `value` into all bits under a mask.
	///
	/// # Parameters
	///
	/// - `value`: The bit that will be directly written by the returned
	///   function.
	///
	/// # Returns
	///
	/// A function which, when applied to a reference and a mask, will write
	/// `value` into memory. If `value` is `false`, then this produces
	/// [`clear_bits`]; if it is `true`, then this produces [`set_bits`].
	///
	/// [`clear_bits`]: Self::clear_bits
	/// [`set_bits`]: Self::set_bits
	#[inline]
	fn get_writers(
		value: bool,
	) -> for<'a> fn(&'a Self, BitMask<<Self as Radium>::Item>) {
		[Self::clear_bits, Self::set_bits][value as usize]
	}

	/// Unconditionally writes a value into a memory location.
	///
	/// # Parameters
	///
	/// - `&self`
	/// - `value`: The new value to write into `*self`.
	///
	/// # Effects
	///
	/// The current value at `*self` is replaced with `value`.
	///
	/// # Safety
	///
	/// The calling context must either have write permissions to the entire
	/// memory element at `*self`, or construct a `value` that does not modify
	/// the bits of `*self` that the caller does not currently own.
	///
	/// As this directly permits the modification of memory outside the logical
	/// ownership of the caller, this method risks behavior that violates the
	/// Rust memory model, even if it may not be technically undefined.
	#[inline]
	unsafe fn store_value(&self, value: <Self as Radium>::Item) {
		self.store(value, Ordering::Relaxed);
	}
}

impl<A> BitAccess for A
where
	A: Debug + Radium,
	<Self as Radium>::Item: BitRegister,
{
}

/** Restricts external code from modifying a memory element to which they lack
exclusive access. This permits the propagation of `&`/`&mut` mutability rules
even to types that may allow shared mutation.
**/
pub trait BitSafe: Debug + Sized {
	/// The register type contained within this wrapper.
	type Mem: BitRegister;

	/// Marks a value as alias-safed.
	fn new(value: Self::Mem) -> Self;

	/// Reads the value out of memory only if a shared reference to the location
	/// can be produced.
	fn load(&self) -> Self::Mem;

	/// Writes a value into memory only if an exclusive reference to the
	/// location can be produced.
	fn store(&mut self, value: Self::Mem);

	/// Removes the alias-safety marker.
	#[inline(always)]
	fn value(self) -> Self::Mem {
		self.load()
	}
}

macro_rules! safe {
	($($t:ident => $cw:ident => $aw:ident => $a:ident),+ $(,)?) => { $(
		/// A wrapper over a [`Cell`] that forbids writing to the location
		/// except through an exclusive reference.
		///
		/// This is necessary in order to enforce [`bitvec`]’s memory model,
		/// which disallows shared mutation to individual bits. [`BitSlice`]s
		/// may produce memory views that use this type in order to ensure that
		/// any other views that alias its memory can be assured that no racing
		/// mutation occurs.
		///
		/// [`BitSlice`]: crate::slice::BitSlice
		/// [`bitvec`]: crate
		/// [`Cell`]: core::cell::Cell
		#[derive(Debug)]
		#[repr(transparent)]
		pub struct $cw {
			inner: Cell<$t>,
		}

		/// A wrapper over an [`atom`] that forbids writing to the location
		/// except through an exclusive reference.
		///
		/// This is necessary in order to enforce [`bitvec`]’s memory model,
		/// which disallows shared mutation to individual bits. [`BitSlice`]s
		/// may produce memory views that use this type in order to ensure that
		/// any other views that alias its memory can be assured that no racing
		/// mutation occurs.
		///
		/// [`BitSlice`]: crate::slice::BitSlice
		/// [`atom`]: core::sync::atomic
		/// [`bitvec`]: crate
		#[derive(Debug)]
		#[repr(transparent)]
		#[cfg(feature = "atomic")]
		pub struct $aw {
			inner: $a,
		}

		impl BitSafe for $cw {
			type Mem = $t;

			#[inline(always)]
			fn new(value: $t) -> Self {
				Self { inner: Cell::new(value) }
			}

			#[inline(always)]
			fn load(&self) -> $t {
				self.inner.get()
			}

			#[inline(always)]
			fn store(&mut self, value: $t) {
				self.inner.set(value);
			}
		}

		#[cfg(feature = "atomic")]
		impl BitSafe for $aw {
			type Mem = $t;

			#[inline(always)]
			fn new(value: $t) -> Self {
				Self { inner: $a::new(value) }
			}

			#[inline(always)]
			fn load(&self) -> $t {
				self.inner.load(Ordering::Relaxed)
			}

			#[inline(always)]
			fn store(&mut self, value: $t) {
				self.inner.store(value, Ordering::Relaxed);
			}
		}
	)+ };
}

safe! {
	u8 => BitSafeCellU8 => BitSafeAtomU8 => AtomicU8,
	u16 => BitSafeCellU16 => BitSafeAtomU16 => AtomicU16,
	u32 => BitSafeCellU32 => BitSafeAtomU32 => AtomicU32,
}

#[cfg(target_pointer_width = "64")]
safe!(u64 => BitSafeCellU64 => BitSafeAtomU64 => AtomicU64);

safe!(usize => BitSafeCellUsize => BitSafeAtomUsize => AtomicUsize);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::prelude::*;

	#[test]
	fn touch_memory() {
		let mut data = 0u8;
		let bits = data.view_bits_mut::<LocalBits>();
		let accessor = unsafe { &*(bits.bitptr().pointer().to_access()) };
		let aliased = unsafe { &*(bits.bitptr().pointer().to_alias()) };

		BitAccess::set_bit::<Lsb0>(accessor, BitIdx::ZERO);
		assert_eq!(accessor.get(), 1);

		BitAccess::set_bits(accessor, BitMask::ALL);
		assert_eq!(accessor.get(), !0);

		BitAccess::clear_bit::<Lsb0>(accessor, BitIdx::ZERO);
		assert_eq!(accessor.get(), !1);

		BitAccess::clear_bits(accessor, BitMask::ALL);
		assert_eq!(accessor.get(), 0);

		BitAccess::invert_bit::<Lsb0>(accessor, BitIdx::ZERO);
		assert_eq!(accessor.get(), 1);
		BitAccess::invert_bits(accessor, BitMask::ALL);
		assert_eq!(accessor.get(), !1);

		assert!(!BitStore::get_bit::<Lsb0>(aliased, BitIdx::ZERO));
		assert_eq!(accessor.get(), !1);

		BitAccess::write_bit::<Lsb0>(accessor, BitIdx::new(1).unwrap(), false);
		assert_eq!(accessor.get(), !3);

		BitAccess::write_bits(accessor, BitMask::ALL, true);
		assert_eq!(accessor.get(), !0);
		BitAccess::write_bits(accessor, Lsb0::mask(BitIdx::new(2), None), false);
		assert_eq!(
			BitStore::get_bits(aliased, Lsb0::mask(BitIdx::new(2), None)),
			0
		);
		assert_eq!(accessor.get(), 3);

		BitAccess::get_writer::<Lsb0>(false)(accessor, BitIdx::ZERO);
		assert_eq!(accessor.get(), 2);

		unsafe {
			BitAccess::store_value(accessor, !1);
		}
		assert_eq!(accessor.get(), !1);
	}

	#[test]
	#[cfg(not(miri))]
	fn sanity_check_prefetch() {
		assert_eq!(
			<Cell<u8> as BitAccess>::get_writer::<Msb0>(false) as *const (),
			<Cell<u8> as BitAccess>::clear_bit::<Msb0> as *const ()
		);

		assert_eq!(
			<Cell<u8> as BitAccess>::get_writer::<Msb0>(true) as *const (),
			<Cell<u8> as BitAccess>::set_bit::<Msb0> as *const ()
		);

		assert_eq!(
			<Cell<u8> as BitAccess>::get_writers(false) as *const (),
			<Cell<u8> as BitAccess>::clear_bits as *const ()
		);

		assert_eq!(
			<Cell<u8> as BitAccess>::get_writers(true) as *const (),
			<Cell<u8> as BitAccess>::set_bits as *const ()
		);
	}
}
