mod common;
use grafton_ndi::{
    BorrowedVideoFrame, PixelFormat, Receiver, ReceiverColorFormat, ReceiverOptions, Sender,
    SenderOptions, NDI,
};
use std::time::{Duration, Instant};

/// A frame in memory the caller owns reaches a receiver as it was built,
/// and the caller has the buffer back between sends.
#[test]
#[ignore = "Requires NDI runtime and loopback networking"]
fn borrowed_frame_reaches_receiver() -> Result<(), grafton_ndi::Error> {
    let ndi = NDI::new()?;
    let sender = Sender::new(
        &ndi,
        &SenderOptions::builder("borrowed send regression")
            .clock_video(false)
            .build(),
    )?;
    let source = common::loopback_source(&ndi, &sender)?;
    let receiver = Receiver::new(
        &ndi,
        &ReceiverOptions::builder(source)
            .color(ReceiverColorFormat::Best)
            .build(),
    )?;

    // P216: a 16-bit Y plane, then interleaved 16-bit Cb/Cr, both full height.
    let (width, height) = (64, 32);
    let mut buffer = vec![0u8; width * 2 * height * 2];
    let metadata = c"<borrowed-send/>";
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let frame =
            BorrowedVideoFrame::builder(&buffer, width as i32, height as i32, PixelFormat::P216)
                .timecode(123_456)
                .metadata(metadata)
                .build()?;
        sender.send_video_borrowed(&frame);
        // The SDK is done with the buffer.
        buffer.fill(0x80);
        if let Some(received) = receiver.video().try_capture(Duration::from_millis(100))? {
            assert_eq!(
                (received.width(), received.height()),
                (width as i32, height as i32)
            );
            assert_eq!(received.pixel_format(), PixelFormat::P216);
            assert_eq!(received.timecode(), 123_456);
            assert_eq!(received.metadata(), Some("<borrowed-send/>"));
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "borrowed frame did not reach loopback receiver"
        );
    }
}
