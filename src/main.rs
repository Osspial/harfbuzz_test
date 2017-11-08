extern crate harfbuzz_sys;
extern crate freetype;
extern crate libc;

use harfbuzz_sys::*;
use freetype::freetype::*;

use libc::*;

use std::{ptr, mem, slice};
use std::io::Read;
use std::fs::File;

fn main() {
    let mut font_buf = vec![];
    File::open("C:/Windows/Fonts/segoeui.ttf").unwrap().read_to_end(&mut font_buf).unwrap();

    unsafe {
        // let mut hb_font = ptr::null_mut();
        // let hb_face = hb_face_create(font_buf.as_ptr(), 0);

        let text = "Hello World!";
        let mut lib = ptr::null_mut();
        let mut ft_face = ptr::null_mut();
        assert_eq!(FT_Error(0), FT_Init_FreeType(&mut lib));
        assert_eq!(FT_Error(0), FT_New_Memory_Face(lib, font_buf.as_ptr(), font_buf.len() as c_int, 0, &mut ft_face));
        assert_eq!(FT_Error(0), FT_Set_Char_Size(ft_face, 0, 16*64, 72, 72));
        println!("{}", FT_Get_Char_Index(ft_face, 'H' as _));

        let size = &*(*ft_face).size;

        let hb_blob = hb_blob_create(font_buf.as_ptr() as *const c_char, font_buf.len() as c_uint, HB_MEMORY_MODE_READONLY, ptr::null_mut(), None);
        let hb_face = hb_face_create(hb_blob, (*ft_face).face_index as c_uint);

        hb_face_set_index(hb_face, (*ft_face).face_index as c_uint);
        hb_face_set_upem(hb_face, (*ft_face).units_per_EM as c_uint);

        let hb_font = hb_font_create(hb_face);

        // let funcs = hb_font_funcs_create();
        // hb_font_funcs_set_font_h_extents_func(funcs, Some(get_font_h_extents), ptr::null_mut(), None);
        // hb_font_funcs_set_nominal_glyph_func(funcs, Some(get_font_nominal_glyph), ptr::null_mut(), None);
        // hb_font_funcs_set_variation_glyph_func(funcs, Some(get_variation_glyph), ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_h_advance_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_v_advance_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_v_origin_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_h_kerning_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_extents_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_contour_point_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_name_func(funcs, None, ptr::null_mut(), None);
        // hb_font_funcs_set_glyph_from_name_func(funcs, None, ptr::null_mut(), None);

        hb_font_set_scale(hb_font,
            ((size.metrics.x_scale as u64 * (*ft_face).units_per_EM as u64 + (1<<15)) >> 16) as i32,
            ((size.metrics.y_scale as u64 * (*ft_face).units_per_EM as u64 + (1<<15)) >> 16) as i32
        );

        let buf = hb_buffer_create();
        hb_buffer_add_utf8(buf, text.as_ptr() as *const c_char, text.len() as i32, 0, text.len() as i32);
        hb_buffer_guess_segment_properties(buf);
        hb_shape(hb_font, buf, ptr::null(), 0);

        let mut glyph_count = 0;
        let glyph_info = hb_buffer_get_glyph_infos(buf, &mut glyph_count);
        let glyph_pos = hb_buffer_get_glyph_positions(buf, &mut glyph_count);

        let glyph_info_slice = slice::from_raw_parts(glyph_info, glyph_count as usize);
        let glyph_pos_slice = slice::from_raw_parts(glyph_pos, glyph_count as usize);

        for (info, pos) in glyph_info_slice.iter().zip(glyph_pos_slice.iter()) {
            // println!("{:?} {:?}", info, pos);
            println!("pos {:?} advance {:?} code {:?}", (pos.x_offset, pos.y_offset), (pos.x_advance, pos.y_advance), info.codepoint);
        }
    }
}

struct FontFuncData {
    ft_face: FT_Face,
    load_flags: c_int,
    symbol: bool
}

unsafe extern "C" fn drop_font_func_data(ffd: *mut c_void) {
    Box::from_raw(ffd as *mut FontFuncData);
}

// These functions are pretty much a direct Rust translation of hb-ft.cc's functions

unsafe extern "C" fn get_font_h_extents(_: *mut hb_font_t, font_data: *mut c_void, metrics: *mut hb_font_extents_t, _: *mut c_void) -> hb_bool_t {
    let ffd = &*(font_data as *const FontFuncData);

    let ft_metrics = &(*(*ffd.ft_face).size).metrics;
    let hb_metrics = &mut *metrics;
    hb_metrics.ascender = ft_metrics.ascender;
    hb_metrics.descender = ft_metrics.descender;
    hb_metrics.line_gap = ft_metrics.height - (ft_metrics.ascender - ft_metrics.descender);

    if ft_metrics.y_scale < 0 {
        hb_metrics.ascender *= -1;
        hb_metrics.descender *= -1;
        hb_metrics.line_gap *= -1;
    }

    1
}

unsafe extern "C" fn get_font_nominal_glyph(_: *mut hb_font_t, font_data: *mut c_void, unicode: hb_codepoint_t, glyph: *mut hb_codepoint_t, _: *mut c_void) -> hb_bool_t {
    let ffd = &*(font_data as *const FontFuncData);
    let mut char_index = FT_Get_Char_Index(ffd.ft_face, unicode);

    if char_index == 0 && ffd.symbol && unicode <= 0x00FF {
        char_index = FT_Get_Char_Index(ffd.ft_face, 0xF000 + unicode);
        if char_index == 0 {
            return 0;
        }
    }

    *glyph = char_index;

    1
}

unsafe extern "C" fn get_variation_glyph(_: *mut hb_font_t, font_data: *mut c_void, unicode: hb_codepoint_t, variation_selector: hb_codepoint_t, glyph: *mut hb_codepoint_t, _: *mut c_void) -> hb_bool_t {
    let ffd = &*(font_data as *const FontFuncData);
    let char_index = FT_Face_GetCharVariantIndex(ffd.ft_face, unicode, variation_selector);

    match char_index {
        0 => 0,
        _ => {
            *glyph = char_index;
            1
        }
    }
}

unsafe extern "C" fn get_h_advance(_: *mut hb_font_t, font_data: *mut c_void, glyph: hb_codepoint_t, _: *mut c_void) -> hb_position_t {
    let ffd = &*(font_data as *const FontFuncData);

    let mut advance = 0;
    match FT_Get_Advance(ffd.ft_face, glyph, ffd.load_flags, &mut advance) {
        FT_Error(0) => {
            if (*(*ffd.ft_face).size).metrics.x_scale < 0 {
                advance *= -1;
            }

            (advance + (1<<9)) >> 10
        },
        _ => 0
    }
}

unsafe extern "C" fn get_v_advance(_: *mut hb_font_t, font_data: *mut c_void, glyph: hb_codepoint_t, _: *mut c_void) -> hb_position_t {
    let ffd = &*(font_data as *const FontFuncData);

    let mut advance = 0;
    match FT_Get_Advance(ffd.ft_face, glyph, ffd.load_flags | FT_LOAD_VERTICAL_LAYOUT as c_int, &mut advance) {
        FT_Error(0) => {
            if (*(*ffd.ft_face).size).metrics.y_scale < 0 {
                advance *= -1;
            }

            (-advance + (1<<9)) >> 10
        },
        _ => 0
    }
}

unsafe extern "C" fn get_glyph_v_origin(_: *mut hb_font_t, font_data: *mut c_void, glyph: hb_codepoint_t, x: *mut hb_position_t, y: *mut hb_position_t, _: *mut c_void) -> hb_bool_t {
    let ffd = &*(font_data as *const FontFuncData);

    match FT_Load_Glyph(ffd.ft_face, glyph, ffd.load_flags) {
        FT_Error(0) => {
            let glyph_metrics = (*(*ffd.ft_face).glyph).metrics;
            *x = glyph_metrics.horiBearingX - glyph_metrics.vertBearingX;
            *y = glyph_metrics.horiBearingY - (-glyph_metrics.vertBearingY);

            if (*(*ffd.ft_face).size).metrics.x_scale < 0 {
                *x *= -1;
            }
            if (*(*ffd.ft_face).size).metrics.y_scale < 0 {
                *y *= -1;
            }

            1
        },
        _ => 0
    }
}

unsafe extern "C" fn get_glyph_h_kerning(_: *mut hb_font_t, font_data: *mut c_void, left_glyph: hb_codepoint_t, right_glyph: hb_codepoint_t, _: *mut c_void) -> hb_position_t {
    let ffd = &*(font_data as *const FontFuncData);

    let mut kerningv = mem::uninitialized();
    let mode = match (*(*ffd.ft_face).size).metrics.x_ppem {
        0 => FT_Kerning_Mode__FT_KERNING_UNFITTED,
        _ => FT_Kerning_Mode__FT_KERNING_DEFAULT
    };
    match FT_Get_Kerning(ffd.ft_face, left_glyph, right_glyph, mode as c_uint, &mut kerningv) {
        FT_Error(0) => kerningv.x,
        _ => 0
    }
}

unsafe extern "C" fn get_glyph_extents(_: *mut hb_font_t, font_data: *mut c_void, glyph: hb_codepoint_t, extents: *mut hb_glyph_extents_t, _: *mut c_void) -> hb_bool_t {
    let ffd = &*(font_data as *const FontFuncData);
    let extents = &mut *extents;

    match FT_Load_Glyph(ffd.ft_face, glyph, ffd.load_flags) {
        FT_Error(0) => {
            let glyph_metrics = (*(*ffd.ft_face).glyph).metrics;
            extents.x_bearing = glyph_metrics.horiBearingX;
            extents.y_bearing = glyph_metrics.horiBearingY;
            extents.width = glyph_metrics.width;
            extents.height = glyph_metrics.height;

            if (*(*ffd.ft_face).size).metrics.x_scale < 0 {
                extents.x_bearing *= -1;
                extents.width *= -1;
            }
            if (*(*ffd.ft_face).size).metrics.y_scale < 0 {
                extents.y_bearing *= -1;
                extents.height *= -1;
            }

            1
        },
        _ => 0
    }
}

// type SnftTable = Vec<FT_Byte>;

// unsafe extern "C" fn ft_reference_table(_: *mut hb_face_t, tag: hb_tag_t, user_data: *mut c_void) -> *mut hb_blob_t {
//     let ft_face = user_data as FT_Face;

//     let mut length = 0;
//     if 0 != FT_Load_Sfnt_Table(ft_face, tag, 0, ptr::null_mut(), &mut length) {
//         return ptr::null_mut();
//     }

//     let mut buf = vec![0; length as usize];
//     let buf_ptr = buf.as_mut_ptr();

//     if 0 != FT_Load_Sfnt_Table(ft_face, tag, 0, buf_ptr, &mut length) {
//         return ptr::null_mut();
//     }

//     let buf_boxed_ptr: *mut SnftTable = Box::into_raw(Box::new(buf));
//     hb_blob_create(buf_ptr as *const c_char, length, HB_MEMORY_MODE_WRITABLE, buf_boxed_ptr as *mut c_void, Some(drop_blob))
// }

// unsafe extern "C" fn drop_blob(bbp: *mut c_void) {
//     Box::from_raw(bbp as *mut SnftTable);
// }
