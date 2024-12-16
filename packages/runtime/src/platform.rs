use alloc::string::String;
use core::ptr;

use vexide::{
    devices::{
        display::{Font, FontFamily, FontSize, Rect, RenderMode, Text},
        math::Point2,
    },
    prelude::*,
};

const LINKED_FILE: *const u32 = 0x7800000 as *const u32;

pub fn read_user_program() -> &'static [u8] {
    unsafe {
        let len = ptr::read_volatile(LINKED_FILE);
        let file_base: *const u8 = LINKED_FILE.offset(1).cast();
        core::slice::from_raw_parts(file_base, len as usize)
    }
}

pub fn flush_serial() {
    while unsafe { vex_sdk::vexSerialWriteFree(1) < 2048 } {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

pub fn draw_error(display: &mut Display, msg: &str) {
    const ERROR_BOX_MARGIN: i16 = 8;
    const ERROR_BOX_PADDING: i16 = 8;
    const LINE_HEIGHT: i16 = 16;
    const LINE_MAX_WIDTH: usize = 56;

    fn draw_text(screen: &mut Display, buffer: &str, line: i16) {
        screen.draw_text(
            &Text::new(
                buffer,
                Font::new(FontSize::SMALL, FontFamily::Monospace),
                Point2 {
                    x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                    y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
                },
            ),
            (255, 255, 255),
            Some(Rgb::new(0, 0, 0)),
        );
    }

    display.set_render_mode(RenderMode::Immediate);

    let error_box_rect = Rect::new(
        Point2 {
            x: ERROR_BOX_MARGIN,
            y: ERROR_BOX_MARGIN,
        },
        Point2 {
            x: Display::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            y: Display::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
        },
    );

    display.fill(&error_box_rect, (255, 0, 0));
    display.stroke(&error_box_rect, (255, 255, 255));

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if character == '\t' {
            buffer += "    ";
        } else if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
            draw_text(display, &buffer, line);
            line += 1;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        draw_text(display, &buffer, line);

        line += 1;
    }
}
