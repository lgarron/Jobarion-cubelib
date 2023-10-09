use crate::coords::coord::Coord;
use crate::cube::{Corner, Edge};
use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

//Corner permutation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPCoord(pub(crate) u16);

//Coordinate representing the position of edges that belong into the FB slice, assuming the UD slice is already correct.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FBSliceUnsortedCoord(pub(crate) u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPOrbitUnsortedCoord(pub(crate) u8);

//Coordinate representing the twist state of HTR corner orbits
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPOrbitTwistCoord(pub(crate) u8);

//Coordinate representing the parity state
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ParityCoord(pub(crate) bool);

//Assuming we already have UD-DR, represents the combination of ParityCoord, CPOrbitUnsortedCoord, CPOrbitTwistCoord and FBSliceUnsortedCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PureHTRDRUDCoord(pub(crate) u16);

//Assuming we already have UD-DR, represents the combination of CPCoord and FBSliceUnsortedCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ImpureHTRDRUDCoord(pub(crate) u32);

impl Coord<40320> for CPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<70> for FBSliceUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FBSliceUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<70> for CPOrbitUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPOrbitUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<3> for CPOrbitTwistCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPOrbitTwistCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2> for ParityCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for ParityCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

//TODO this should use 'impl const' once it's stable
pub const PURE_HTRDRUD_SIZE: usize = 70 * 70 * 6;
impl Coord<PURE_HTRDRUD_SIZE> for PureHTRDRUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for PureHTRDRUDCoord {
    fn into(self) -> usize {
        self.val()
    }
}

//TODO this should use 'impl const' once it's stable
pub const IMPURE_HTRDRUD_SIZE: usize = 70 * 40320;
impl Coord<{ IMPURE_HTRDRUD_SIZE }> for ImpureHTRDRUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for ImpureHTRDRUDCoord {
    fn into(self) -> usize {
        self.val()
    }
}

pub type HTRDRUDCoord = ImpureHTRDRUDCoord;
pub const HTRDRUD_SIZE: usize = IMPURE_HTRDRUD_SIZE;


impl From<&CornerCubieCube> for CPCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_cpcoord(value) }
    }
}

impl From<&[Corner; 8]> for CPCoord {
    fn from(value: &[Corner; 8]) -> Self {
        let mut cp = 0_u16;
        let factorial = [1, 2, 6, 24, 120, 720, 5040];

        for i in 1..8 {
            let mut higher = 0;
            for j in 0..i {
                if value[i].id < value[j].id {
                    higher += 1;
                }
            }
            cp += factorial[i - 1] * higher;
        }
        CPCoord(cp)
    }
}

impl From<&EdgeCubieCube> for FBSliceUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_fbslice_unsorted_coord(value) }
    }
}

impl From<&CornerCubieCube> for CPOrbitUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_cp_orbit_unsorted_coord(value) }
    }
}

impl From<&CornerCubieCube> for CPOrbitTwistCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_cp_orbit_twist_parity_coord(value) }
    }
}

impl From<&CornerCubieCube> for ParityCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_parity_coord(value) }
    }
}

impl From<&CubieCube> for PureHTRDRUDCoord {
    fn from(value: &CubieCube) -> Self {
        let ep_fbslice_coord = FBSliceUnsortedCoord::from(&value.edges).val();
        let cp_orbit_coord = CPOrbitUnsortedCoord::from(&value.corners).val();
        let cp_orbit_twist = CPOrbitTwistCoord::from(&value.corners).val();
        let parity = ParityCoord::from(&value.corners).val();

        let val = parity
            + cp_orbit_twist * ParityCoord::size()
            + cp_orbit_coord * ParityCoord::size() * CPOrbitTwistCoord::size()
            + ep_fbslice_coord
            * ParityCoord::size()
            * CPOrbitTwistCoord::size()
            * CPOrbitUnsortedCoord::size();
        Self(val as u16)
    }
}

impl From<&CubieCube> for ImpureHTRDRUDCoord {
    fn from(value: &CubieCube) -> Self {
        let ep_fbslice_coord = FBSliceUnsortedCoord::from(&value.edges).val();
        let cp = CPCoord::from(&value.corners).val();

        let val = cp + ep_fbslice_coord * CPCoord::size();
        Self(val as u32)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_castps_si128, _mm_castsi128_ps, _mm_cmpeq_epi8, _mm_cmplt_epi8, _mm_extract_epi16, _mm_extract_epi64, _mm_hadd_epi16, _mm_hadd_epi32, _mm_movemask_epi8, _mm_mullo_epi16, _mm_or_si128, _mm_permute_ps, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_slli_epi32, _mm_slli_epi64, _mm_slli_si128, _mm_srli_epi32, _mm_sub_epi8, _mm_xor_si128};
    use crate::alignment::avx2::C;
    use crate::coords::dr::{COUDCoord, UDSliceUnsortedCoord};
    use crate::coords::eo::{EOCoordAll, EOCoordFB, EOCoordLR, EOCoordUD};
    use crate::coords::htr::{CPCoord, CPOrbitTwistCoord, CPOrbitUnsortedCoord, FBSliceUnsortedCoord, ParityCoord};
    use crate::cubie::{CornerCubieCube, EdgeCubieCube};

    const UD_SLICE_BINOM_0_ARR: [u8; 16] = [
        b(0, 0),
        b(0, 1),
        b(0, 2),
        b(0, 3),
        b(1, 0),
        b(1, 1),
        b(1, 2),
        b(1, 3),
        b(2, 0),
        b(2, 1),
        b(2, 2),
        b(2, 3),
        b(3, 0),
        b(3, 1),
        b(3, 2),
        b(3, 3),
    ];
    const UD_SLICE_BINOM_1_ARR: [u8; 16] = [
        b(4, 0),
        b(4, 1),
        b(4, 2),
        b(4, 3),
        b(5, 0),
        b(5, 1),
        b(5, 2),
        b(5, 3),
        b(6, 0),
        b(6, 1),
        b(6, 2),
        b(6, 3),
        b(7, 0),
        b(7, 1),
        b(7, 2),
        b(7, 3),
    ];
    const UD_SLICE_BINOM_2_ARR: [u8; 16] = [
        b(8, 0),
        b(8, 1),
        b(8, 2),
        b(8, 3),
        b(9, 0),
        b(9, 1),
        b(9, 2),
        b(9, 3),
        b(10, 0),
        b(10, 1),
        b(10, 2),
        b(10, 3),
        b(11, 0),
        b(11, 1),
        b(11, 2),
        b(11, 3),
    ];

    const UD_SLICE_BINOM_0: __m128i = unsafe {
        C {
            a_u8: UD_SLICE_BINOM_0_ARR,
        }
            .a
    };
    const UD_SLICE_BINOM_1: __m128i = unsafe {
        C {
            a_u8: UD_SLICE_BINOM_1_ARR,
        }
            .a
    };

    const ORBIT_STATE_LUT: [u8; 56] = [
        //  x, F, L, U, U, L, F, x
        3, 3, 3, 3, 3, 3, 3, 3, //x
        3, 0, 2, 1, 1, 2, 0, 3, //F
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 0, 2, 1, 1, 2, 0, 3, //F
        // 3, 3, 3, 3, 3, 3, 3, 3,  //x
    ];



    const CP_ORBIT_SHUFFLE_BLOCK_0: [__m128i; 16] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0000
        unsafe { C { a_u8: [1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0001
        unsafe { C { a_u8: [0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0010
        unsafe { C { a_u8: [2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0011
        unsafe { C { a_u8: [0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0100
        unsafe { C { a_u8: [1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0101
        unsafe { C { a_u8: [0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0110
        unsafe { C { a_u8: [3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0111
        unsafe { C { a_u8: [0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1000
        unsafe { C { a_u8: [1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1001
        unsafe { C { a_u8: [0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1010
        unsafe { C { a_u8: [2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1011
        unsafe { C { a_u8: [0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1100
        unsafe { C { a_u8: [1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1101
        unsafe { C { a_u8: [0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1110
        unsafe { C { a_u8: [0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1111
    ];

    const CP_ORBIT_SHUFFLE_BLOCK_1: [__m128i; 16] = [
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0000
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF] }.a },//0001
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF] }.a },//0010
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF] }.a },//0011
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF] }.a },//0100
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF] }.a },//0101
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF] }.a },//0110
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF] }.a },//0111
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF] }.a },//1000
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF] }.a },//1001
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF] }.a },//1010
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF] }.a },//1011
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF] }.a },//1100
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF] }.a },//1101
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF] }.a },//1110
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7] }.a },//1111
    ];

    const CP_ORBIT_SHUFFLE_GAP_0: [__m128i; 5] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 1, 2, 4, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 1, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
    ];

    const CP_ORBIT_SHUFFLE_GAP_1: [__m128i; 5] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 10, 12, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 12, 13, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 12, 13, 14, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 12, 13, 14, 15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
    ];

    unsafe fn arrange_orbit_corners(value: __m128i) -> __m128i {
        let corners_with_marker = _mm_or_si128(
            value,
            _mm_set_epi8(-1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        );
        let ud_corners = _mm_movemask_epi8(_mm_slli_epi32::<2>(value)) as usize;
        let block_0 = ud_corners & 0xF;
        let block_1 = (ud_corners >> 4) & 0xF;

        let ud_corners_sorted_gaps = _mm_or_si128(
            _mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_0[block_0]),
            _mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_1[block_1]),
        );

        let gaps = _mm_and_si128(
            _mm_cmpeq_epi8(ud_corners_sorted_gaps, _mm_set1_epi8(-1)),
            _mm_set1_epi8(1),
        );
        let gap_sizes = _mm_sad_epu8(gaps, _mm_set1_epi8(0));

        let gap_sizes = _mm_extract_epi64::<0>(_mm_shuffle_epi32::<0b11111000>(gap_sizes)) as usize;
        let gap_0 = gap_sizes & 0xF;
        let gap_1 = (gap_sizes >> 32) & 0xF;

        _mm_shuffle_epi8(
            _mm_shuffle_epi8(ud_corners_sorted_gaps, CP_ORBIT_SHUFFLE_GAP_0[gap_0]),
            CP_ORBIT_SHUFFLE_GAP_1[gap_1],
        )
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_fbslice_unsorted_coord(
        value: &EdgeCubieCube,
    ) -> FBSliceUnsortedCoord {
        let fb_slice_edges = _mm_shuffle_epi8(
            _mm_set_epi8(0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0),
            _mm_and_si128(_mm_srli_epi32::<4>(value.0), _mm_set1_epi8(0x0F)),
        );
        let fb_slice_edges = _mm_shuffle_epi8(
            fb_slice_edges,
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 11, 9, 3, 1, 10, 8, 2, 0),
        );

        FBSliceUnsortedCoord(unsorted_coord_4_4_split(fb_slice_edges))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cp_orbit_unsorted_coord(
        value: &CornerCubieCube,
    ) -> CPOrbitUnsortedCoord {
        let orbit_corners = _mm_srli_epi32::<5>(_mm_and_si128(value.0, _mm_set1_epi8(0b00100000)));
        let orbit_corners = _mm_shuffle_epi8(
            orbit_corners,
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 7, 5, 3, 1, 6, 4, 2, 0),
        );
        CPOrbitUnsortedCoord(unsorted_coord_4_4_split(orbit_corners))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsorted_coord_4_4_split(value: __m128i) -> u8 {
        let marked = value;
        let unmarked = _mm_cmpeq_epi8(marked, _mm_set1_epi8(0));

        let c0123 = _mm_shuffle_epi8(
            marked,
            _mm_set_epi8(3, -1, -1, -1, 2, 2, -1, -1, 1, 1, 1, -1, 0, 0, 0, 0),
        );
        let c4567 = _mm_shuffle_epi8(
            marked,
            _mm_set_epi8(7, -1, -1, -1, 6, 6, -1, -1, 5, 5, 5, -1, 4, 4, 4, 4),
        );

        let hadd = _mm_hadd_epi32(c0123, c4567);
        let hadd = _mm_hadd_epi32(hadd, _mm_set1_epi8(0));
        let hadd = _mm_add_epi8(
            hadd,
            _mm_shuffle_epi8(
                hadd,
                _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, -1, -1, -1, -1),
            ),
        );
        let hadd = _mm_and_si128(hadd, unmarked);

        let lut_index = _mm_and_si128(
            _mm_sub_epi8(hadd, _mm_set1_epi8(1)),
            _mm_set1_epi8(0b10001111_u8 as i8),
        );
        let lut_index = _mm_add_epi8(
            lut_index,
            _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 12, 8, 4, 0, 12, 8, 4, 0),
        );

        let binom0123 = _mm_and_si128(
            _mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index),
            _mm_set_epi32(0, 0, 0, -1),
        );
        let binom4567 = _mm_and_si128(
            _mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index),
            _mm_set_epi32(0, 0, -1, 0),
        );

        let sum = _mm_sad_epu8(_mm_or_si128(binom0123, binom4567), _mm_set1_epi8(0));

        _mm_extract_epi16::<0>(sum) as u8
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
        let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        _mm_extract_epi16::<0>(sum) as u16
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cpcoord(value: &CornerCubieCube) -> CPCoord {
        let cp_values = _mm_and_si128(_mm_srli_epi32::<5>(value.0), _mm_set1_epi8(0b111));

        //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
        let values_67 = _mm_shuffle_epi8(
            cp_values,
            _mm_set_epi8(-1, -1, 7, -1, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6),
        );
        let values_2345 = _mm_shuffle_epi8(
            cp_values,
            _mm_set_epi8(5, 4, -1, -1, 5, 4, 3, -1, 5, 4, 3, 2, 5, 4, 3, 2),
        );
        let values_15 = _mm_shuffle_epi8(cp_values, _mm_set_epi64x(5, 1));

        let higher_left_67 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_67,
                _mm_shuffle_epi8(
                    cp_values,
                    _mm_set_epi8(-1, -1, 6, -1, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_2345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_2345,
                _mm_shuffle_epi8(
                    cp_values,
                    _mm_set_epi8(3, 3, -1, -1, 2, 2, 2, -1, 1, 1, 1, 1, 0, 0, 0, 0),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_15 = _mm_and_si128(
            _mm_cmplt_epi8(values_15, _mm_shuffle_epi8(cp_values, _mm_set_epi64x(4, 0))),
            _mm_set1_epi8(1),
        );

        let hsum = _mm_hadd_epi32(higher_left_2345, higher_left_67);
        let hsum = _mm_hadd_epi32(hsum, higher_left_15);
        let hsum = _mm_shuffle_epi8(
            hsum,
            _mm_set_epi8(-1, 7, -1, 5, 6, 12, 4, 3, -1, -1, 2, 1, -1, -1, 0, 8),
        );
        let hsum = _mm_hadd_epi16(hsum, _mm_set1_epi8(0));
        let hsum = _mm_shuffle_epi8(
            hsum,
            _mm_set_epi8(-1, -1, -1, 6, -1, 5, -1, 4, -1, 3, -1, 2, -1, 1, -1, 0),
        );
        let factorials = _mm_set_epi16(0, 5040, 720, 120, 24, 6, 2, 1);
        let prod = _mm_mullo_epi16(hsum, factorials);

        CPCoord(hsum_epi16_sse3(prod))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_cp_orbit_twist_parity_coord(
        cube: &CornerCubieCube,
    ) -> CPOrbitTwistCoord {
        // println!("{:?}", cube.0);
        let orbit_corners = arrange_orbit_corners(cube.0);
        let relevant_corners = _mm_shuffle_epi8(
            orbit_corners,
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 5, 4, 2, 1, 0),
        );

        // let orbit_corners = cube.0;
        // let relevant_corners = _mm_shuffle_epi8(orbit_corners, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 3, 1, 4, 2, 0));

        // println!("{:?}", orbit_corners);

        let ud = _mm_movemask_epi8(relevant_corners);
        // let fb = _mm_movemask_epi8(_mm_add_epi8(relevant_corners, _mm_set1_epi8(0b01000000)));
        // let lr = _mm_movemask_epi8(_mm_slli_epi32::<1>(_mm_add_epi8(relevant_corners, _mm_set1_epi8(0b00100000))));

        // println!("{ud}");

        let ud_twist = ORBIT_STATE_LUT[ud as usize];
        // let fb_twist = ORBIT_STATE_LUT[fb as usize];
        // let lr_twist = ORBIT_STATE_LUT[lr as usize];

        // println!("{:?}", ud_twist);
        // println!("{:?}", fb_twist);
        // println!("{:?}", lr_twist);

        CPOrbitTwistCoord(ud_twist)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_parity_coord(cube: &CornerCubieCube) -> ParityCoord {
        let values_12345 = _mm_shuffle_epi8(
            cube.0,
            _mm_set_epi8(-1, 5, 5, 5, 5, 5, 4, 4, 4, 4, 3, 3, 3, 2, 2, 1),
        );
        let values_67 = _mm_shuffle_epi8(
            cube.0,
            _mm_set_epi8(-1, -1, -1, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6),
        );

        let higher_left_12345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_12345,
                _mm_shuffle_epi8(
                    cube.0,
                    _mm_set_epi8(-1, 4, 3, 2, 1, 0, 3, 2, 1, 0, 2, 1, 0, 1, 0, 0),
                ),
            ),
            _mm_set1_epi8(1),
        );

        let higher_left_67 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_67,
                _mm_shuffle_epi8(
                    cube.0,
                    _mm_set_epi8(-1, -1, -1, 6, 5, 4, 3, 2, 1, 0, 5, 4, 3, 2, 1, 0),
                ),
            ),
            _mm_set1_epi8(1),
        );

        let parity = _mm_xor_si128(higher_left_12345, higher_left_67);
        let parity = _mm_sad_epu8(parity, _mm_set1_epi8(0));
        let parity = _mm_extract_epi64::<0>(_mm_castps_si128(_mm_permute_ps::<0b00001000>(
            _mm_castsi128_ps(parity),
        )));
        let parity = (parity ^ (parity >> 32)) & 1;

        ParityCoord(parity == 1)
    }


    const FACTORIAL: [u32; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
    ];

    const fn b(n: u8, k: u8) -> u8 {
        if n == 0 || n < k {
            return 0;
        }
        (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
    }
}