use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

#[cfg(target_arch = "wasm32")]
use {
    js_sys,
    leptos::wasm_bindgen::prelude::*,
    leptos::wasm_bindgen::JsCast,
    leptos::web_sys::{
        CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, ImageData, MediaStream,
        MediaStreamConstraints,
    },
    wasm_bindgen_futures::JsFuture,
};

#[component]
pub fn QrScanner(
    #[prop(into)] on_scan: Callback<String>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let video_ref = NodeRef::<leptos::html::Video>::new();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let viewport_ref = NodeRef::<leptos::html::Div>::new();
    let (scanning, set_scanning) = signal(false);
    let (error, set_error) = signal(None::<String>);
    
    // Zoom functionality state
    let (zoom_level, set_zoom_level) = signal(1.0_f64);
    
    const MIN_ZOOM: f64 = 1.0;
    const MAX_ZOOM: f64 = 6.0;
    const ZOOM_STEP: f64 = 0.25;

    // Start camera when component mounts
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        spawn_local(async move {
            match start_camera(video_ref, set_error).await {
                Ok(_) => {
                    set_scanning.set(true);
                    start_scanning(canvas_ref, video_ref, on_scan, set_scanning);
                }
                Err(e) => {
                    set_error.set(Some(format!("Camera access failed: {:?}", e)));
                }
            }
        });
    });

    let handle_close = move |_: leptos::ev::MouseEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(video_el) = video_ref.get() {
                let video: HtmlVideoElement = video_el.clone().into();
                if let Some(stream) = video.src_object() {
                    if let Ok(media_stream) = stream.dyn_into::<MediaStream>() {
                        stop_stream(&media_stream);
                    }
                }
            }
        }
        on_close.run(());
    };

    // Zoom button handlers - work for all builds
    let handle_zoom_in = move |_: leptos::ev::MouseEvent| {
        let current = zoom_level.get();
        if current < MAX_ZOOM {
            let new_zoom = (current + ZOOM_STEP).min(MAX_ZOOM);
            set_zoom_level.set(new_zoom);
            leptos::logging::log!("Zoom in: {} -> {}", current, new_zoom);
        }
    };

    let handle_zoom_out = move |_: leptos::ev::MouseEvent| {
        let current = zoom_level.get();
        if current > MIN_ZOOM {
            let new_zoom = (current - ZOOM_STEP).max(MIN_ZOOM);
            set_zoom_level.set(new_zoom);
            leptos::logging::log!("Zoom out: {} -> {}", current, new_zoom);
        }
    };

    #[cfg(target_arch = "wasm32")]
    let content = view! {
        <div class="qr-scanner-overlay">
            <div class="qr-scanner-container">
                <div class="qr-scanner-header">
                    <h2>"Scan QR Code"</h2>
                    <button class="qr-close-btn" on:click=handle_close>
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>

                <div 
                    class="qr-scanner-viewport"
                    node_ref=viewport_ref
                    style="overflow: hidden;"
                >
                    <video
                        node_ref=video_ref
                        class="qr-video"
                        autoplay=true
                        playsinline=true
                        muted=true
                        style=move || format!("transform: scale({}); transform-origin: center center; transition: transform 0.1s ease-out;", zoom_level.get())
                    />
                    <canvas node_ref=canvas_ref class="qr-canvas" style="display: none;" />

                    <div class="qr-scanner-frame">
                        <div class="qr-corner qr-corner-tl"></div>
                        <div class="qr-corner qr-corner-tr"></div>
                        <div class="qr-corner qr-corner-bl"></div>
                        <div class="qr-corner qr-corner-br"></div>
                    </div>
                </div>

                {move || error.get().map(|err| view! {
                    <div class="qr-error">
                        <p>{err}</p>
                    </div>
                })}

                {/* Zoom indicator */}
                {move || {
                    let current_zoom = zoom_level.get();
                    if current_zoom > 1.01 {
                        view! {
                            <div class="qr-zoom-indicator-badge">
                                <span>{format!("{:.1}x", current_zoom)}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}

                {/* Zoom controls */}
                <div class="qr-zoom-controls">
                    <button class="qr-zoom-btn" on:click=handle_zoom_out>
                        "−"
                    </button>
                    <span class="qr-zoom-level">
                        {move || format!("{:.2}x", zoom_level.get())}
                    </span>
                    <button class="qr-zoom-btn" on:click=handle_zoom_in>
                        "+"
                    </button>
                </div>

                <div class="qr-instructions">
                    <p>"Position the QR code within the frame"</p>
                    {move || if scanning.get() {
                        view! { <p class="qr-status-scanning">"Scanning..."</p> }.into_any()
                    } else {
                        view! { <p class="qr-status-idle">"Ready"</p> }.into_any()
                    }}
                    <p class="qr-zoom-help">"Use + and - buttons to zoom"</p>
                </div>
            </div>
        </div>
    };

    #[cfg(not(target_arch = "wasm32"))]
    let content = {
        // Use variables to avoid unused warnings
        let _ = video_ref;
        let _ = canvas_ref;
        let _ = viewport_ref;
        let _ = scanning;
        let _ = set_scanning;
        let _ = error;
        let _ = set_error;
        let _ = zoom_level;
        let _ = set_zoom_level;
        let _ = handle_zoom_in;
        let _ = handle_zoom_out;
        let _ = handle_close;
        let _ = on_scan;
        
        view! {
            <div class="qr-scanner-overlay">
                <div class="qr-scanner-container">
                    <div class="qr-scanner-header">
                        <h2>"QR Scanner Not Available"</h2>
                        <button class="qr-close-btn" on:click=move |_| on_close.run(())>
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                <line x1="6" y1="6" x2="18" y2="18"></line>
                            </svg>
                        </button>
                    </div>
                    <div class="qr-instructions">
                        <p>"QR scanning is only available in the browser"</p>
                    </div>
                </div>
            </div>
        }
    };

    content
}

#[cfg(target_arch = "wasm32")]
fn calculate_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(target_arch = "wasm32")]
async fn start_camera(
    video_ref: NodeRef<leptos::html::Video>,
    set_error: WriteSignal<Option<String>>,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let media_devices = navigator
        .media_devices()
        .map_err(|_| "Media devices not supported")?;

    let constraints = MediaStreamConstraints::new();

    // Create video constraints for rear camera with high resolution
    let video_constraints = js_sys::Object::new();
    js_sys::Reflect::set(
        &video_constraints,
        &JsValue::from_str("facingMode"),
        &JsValue::from_str("environment"),
    )?;
    
    js_sys::Reflect::set(
        &video_constraints,
        &JsValue::from_str("width"),
        &JsValue::from_f64(1920.0),
    )?;
    js_sys::Reflect::set(
        &video_constraints,
        &JsValue::from_str("height"),
        &JsValue::from_f64(1080.0),
    )?;

    constraints.set_video(&video_constraints);
    constraints.set_audio(&JsValue::FALSE);

    let promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|_| "Failed to get user media")?;

    let stream = JsFuture::from(promise).await?;
    let media_stream: MediaStream = stream.dyn_into()?;

    if let Some(video_element) = video_ref.get() {
        let video: HtmlVideoElement = video_element.clone().into();
        video.set_src_object(Some(&media_stream));
        let _ = video.play().map_err(|e| {
            set_error.set(Some("Failed to play video".to_string()));
            e
        });
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn stop_stream(stream: &MediaStream) {
    let tracks = stream.get_tracks();
    for i in 0..tracks.length() {
        let track = tracks.get(i);
        if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
            track.stop();
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn start_scanning(
    canvas_ref: NodeRef<leptos::html::Canvas>,
    video_ref: NodeRef<leptos::html::Video>,
    on_scan: Callback<String>,
    set_scanning: WriteSignal<bool>,
) {
    use gloo::timers::callback::Interval;

    spawn_local(async move {
        let _interval = Interval::new(500, move || {
            if let (Some(canvas_el), Some(video_el)) = (canvas_ref.get(), video_ref.get()) {
                let canvas: HtmlCanvasElement = canvas_el.clone().into();
                let video: HtmlVideoElement = video_el.clone().into();

                if video.ready_state() >= 2 {
                    let width = video.video_width();
                    let height = video.video_height();

                    if width > 0 && height > 0 {
                        canvas.set_width(width);
                        canvas.set_height(height);

                        if let Ok(Some(context)) = canvas.get_context("2d") {
                            if let Ok(ctx) = context.dyn_into::<CanvasRenderingContext2d>() {
                                let _ = ctx.draw_image_with_html_video_element(&video, 0.0, 0.0);

                                if let Ok(image_data) =
                                    ctx.get_image_data(0.0, 0.0, width as f64, height as f64)
                                {
                                    if let Some(qr_data) = decode_qr(&image_data) {
                                        set_scanning.set(false);
                                        on_scan.run(qr_data);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        std::future::pending::<()>().await;
    });
}

#[cfg(target_arch = "wasm32")]
fn decode_qr(image_data: &ImageData) -> Option<String> {
    let width = image_data.width();
    let height = image_data.height();
    let data = image_data.data();

    let mut gray_data = Vec::with_capacity((width * height) as usize);
    for i in (0..data.len()).step_by(4) {
        let r = data[i] as u32;
        let g = data[i + 1] as u32;
        let b = data[i + 2] as u32;
        let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
        gray_data.push(gray);
    }

    let mut img =
        rqrr::PreparedImage::prepare_from_greyscale(width as usize, height as usize, |x, y| {
            gray_data[y * width as usize + x]
        });

    let grids = img.detect_grids();
    for grid in grids {
        if let Ok((_meta, content)) = grid.decode() {
            return Some(content);
        }
    }

    None
}