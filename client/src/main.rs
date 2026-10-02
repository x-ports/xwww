use std::{str::FromStr, time::Duration};

use clap::Parser;
use common::cache;
use common::ipc::{self, Answer, BgInfo, IpcSocket, RequestSend};
use common::mmap::Mmap;

mod imgproc;
use imgproc::*;

mod cli;
use cli::{CliImage, CropGravity, Filter, ResizeStrategy, Xwww};

mod effects;

mod palette;
mod palette_source;
mod screenshot;
mod slideshow;

#[cfg(feature = "scene")]
mod scene;

#[cfg(feature = "video")]
mod video;

fn main() -> Result<(), String> {
    common::log::init(common::log::Filter::Trace);
    let xwww = Xwww::parse();

    // These commands do not follow the standard request/answer flow.
    match &xwww {
        Xwww::Palette(p) => return palette::run(p),
        Xwww::Slideshow(s) => return slideshow::run(s),
        Xwww::Random(r) => return slideshow::run_random(r),
        Xwww::Screenshot(s) => return screenshot::run(s),
        Xwww::Scene(scene_args) => {
            #[cfg(feature = "scene")]
            {
                return run_scene_cli(scene_args);
            }
            #[cfg(not(feature = "scene"))]
            {
                let _ = scene_args;
                return Err(
                    "this build of xwww has no scene support (rebuild with `--features scene`)"
                        .to_string(),
                );
            }
        }
        _ => {}
    }

    let all = match &xwww {
        Xwww::Clear(clear) => clear.all,
        Xwww::Restore(restore) => restore.all,
        Xwww::ClearCache => {
            return cache::clean().map_err(|e| format!("failed to clean the cache: {e}"));
        }
        Xwww::Img(img) => img.all,
        Xwww::Toggle(toggle) => toggle.all,
        Xwww::Pause(pause) => pause.all,
        Xwww::Unpause(unpause) => unpause.all,
        Xwww::Kill(kill) => kill.all,
        Xwww::Query(query) => query.all,
        Xwww::Palette(_)
        | Xwww::Slideshow(_)
        | Xwww::Random(_)
        | Xwww::Screenshot(_)
        | Xwww::Scene(_) => unreachable!(),
    };

    let namespaces = if all {
        IpcSocket::all_namespaces().map_err(|e| e.to_string())?
    } else {
        match &xwww {
            Xwww::Clear(clear) => clear.namespace.clone(),
            Xwww::Restore(restore) => restore.namespace.clone(),
            Xwww::ClearCache => {
                return cache::clean().map_err(|e| format!("failed to clean the cache: {e}"));
            }
            Xwww::Img(img) => img.namespace.clone(),
            Xwww::Toggle(toggle) => toggle.namespace.clone(),
            Xwww::Pause(pause) => pause.namespace.clone(),
            Xwww::Unpause(unpause) => unpause.namespace.clone(),
            Xwww::Kill(kill) => kill.namespace.clone(),
            Xwww::Query(query) => query.namespace.clone(),
            Xwww::Palette(_)
            | Xwww::Slideshow(_)
            | Xwww::Random(_)
            | Xwww::Screenshot(_)
            | Xwww::Scene(_) => unreachable!(),
        }
    };

    let mut infos = Vec::new();
    for namespace in &namespaces {
        let socket = IpcSocket::client(namespace).map_err(|err| err.to_string())?;
        loop {
            RequestSend::Ping.send(&socket).map_err(|e| e.to_string())?;
            let bytes = socket.recv().map_err(|err| err.to_string())?;
            let answer = Answer::receive(bytes);
            if let Answer::Ping(configured) = answer {
                if configured {
                    break;
                }
            } else {
                return Err("Daemon did not return Answer::Ping, as expected".to_string());
            }
            std::thread::sleep(Duration::from_millis(1));
        }

        if let Some(info) = process_xwww_args(&xwww, namespace)? {
            infos.push(info);
        }
    }

    if !infos.is_empty() {
        if let Xwww::Query(query) = xwww
            && query.json
        {
            use jzon::{JsonValue, object, stringify_pretty};
            let mut buf = String::new();
            for (namespace, infos) in namespaces.iter().zip(infos) {
                let mut arr = JsonValue::new_array();
                for info in infos {
                    let displaying = match info.img {
                        ipc::BgImg::Color(color) => {
                            object! { color: format!("#{:x}", u32::from_ne_bytes(color)) }
                        }
                        ipc::BgImg::Img(img) => {
                            object! { image: img.as_ref() }
                        }
                    };
                    _ = arr.push(object! {
                        name: info.name.as_ref(),
                        width: info.dim.0,
                        height: info.dim.1,
                        scale: info.scale_factor.to_f32(),
                        displaying: displaying
                    });
                }
                buf = format!("{buf}\n\"{namespace}\": {},", stringify_pretty(arr, 4));
            }
            buf.pop(); // delete trailing comma
            println!("{{{buf}\n}}");
        } else {
            for (namespace, infos) in namespaces.iter().zip(infos) {
                for info in infos {
                    println!("{namespace}: {info}");
                }
            }
        }
    }
    Ok(())
}

fn process_xwww_args(args: &Xwww, namespace: &str) -> Result<Option<Box<[BgInfo]>>, String> {
    let request = match make_request(args, namespace)? {
        Some(request) => request,
        None => return Ok(None),
    };
    let socket = IpcSocket::client(namespace).map_err(|err| err.to_string())?;
    request.send(&socket).map_err(|e| e.to_string())?;
    let bytes = socket.recv().map_err(|err| err.to_string())?;
    drop(socket);
    match Answer::receive(bytes) {
        Answer::Info(infos) => {
            if let Xwww::Query(_) = args {
                return Ok(Some(infos));
            }
        }
        Answer::Ok => {
            if let Xwww::Kill(_) = args {
                #[cfg(debug_assertions)]
                let tries = 20;
                #[cfg(not(debug_assertions))]
                let tries = 10;
                let path = IpcSocket::path(namespace);
                for _ in 0..tries {
                    if rustix::fs::access(&path, rustix::fs::Access::EXISTS).is_err() {
                        return Ok(None);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                return Err(format!(
                    "Could not confirm socket deletion at: {}",
                    path.display()
                ));
            }
        }
        Answer::Ping(_) => {
            return Ok(None);
        }
        Answer::Screenshot(_) => {
            return Err("unexpected screenshot answer".to_string());
        }
    }
    Ok(None)
}

fn make_request(args: &Xwww, namespace: &str) -> Result<Option<RequestSend>, String> {
    match args {
        Xwww::Clear(c) => {
            let (format, _, _) = get_format_dims_and_outputs(&[], namespace)?;
            let mut color = c.color;
            if format.must_swap_r_and_b_channels() {
                color.swap(0, 2);
            }
            let clear = ipc::ClearSend {
                color,
                outputs: split_cmdline_outputs(&c.outputs),
            };
            Ok(Some(RequestSend::Clear(
                clear.create_request().map_err(|e| e.to_string())?,
            )))
        }
        Xwww::Restore(restore) => {
            let requested_outputs = split_cmdline_outputs(&restore.outputs);
            restore_from_cache(&requested_outputs, namespace)?;
            Ok(None)
        }
        Xwww::ClearCache => unreachable!("there is no request for clear-cache"),
        Xwww::Img(img) => {
            let requested_outputs = split_cmdline_outputs(&img.outputs);
            let (format, dims, outputs) =
                get_format_dims_and_outputs(&requested_outputs, namespace)?;
            // let imgbuf = ImgBuf::new(&img.path)?;

            let img_request = make_img_request(
                img,
                namespace,
                &dims,
                format,
                &outputs,
                img.outputs.is_empty(),
            )?;

            Ok(Some(RequestSend::Img(img_request)))
        }
        Xwww::Toggle(_) => Ok(Some(RequestSend::Toggle)),
        Xwww::Pause(_) => Ok(Some(RequestSend::Pause)),
        Xwww::Unpause(_) => Ok(Some(RequestSend::Unpause)),
        Xwww::Kill(_) => Ok(Some(RequestSend::Kill)),
        Xwww::Query(_) => Ok(Some(RequestSend::Query)),
        Xwww::Palette(_)
        | Xwww::Slideshow(_)
        | Xwww::Random(_)
        | Xwww::Screenshot(_)
        | Xwww::Scene(_) => unreachable!(),
    }
}

fn make_img_request(
    img: &cli::Img,
    namespace: &str,
    dims: &[(u32, u32)],
    pixel_format: ipc::PixelFormat,
    outputs: &[Vec<String>],
    update_cached_disconnected_outputs: bool,
) -> Result<Mmap, String> {
    let transition = make_transition(&img.transition_args());

    let palette_stops = img
        .map_palette
        .as_deref()
        .map(|spec| {
            palette_source::ScenePalette::load(spec)
                .map(|palette| palette.gradient_stops())
                .map_err(|e| format!("failed to load palette '{spec}': {e}"))
        })
        .transpose()?;

    let mut img_req_builder = ipc::ImageRequestBuilder::new(transition)
        .map_err(|e| format!("failed to create ImageRequestBuilder: {e}"))?;

    let use_cache = !img.no_cache;
    let filter = img.filter.as_str();
    let resize = img.resize.as_str();
    let crop_gravity_str = img.crop_gravity.map(|v| v.as_str());

    let cache_path;

    match &img.image {
        CliImage::Color(color) => {
            let color_path = format!(
                "0x{:02x}{:02x}{:02x}{:02x}",
                color[0], color[1], color[2], color[3]
            );

            cache_path = color_path.clone();

            for (&dim, outputs) in dims.iter().zip(outputs) {
                img_req_builder.push(
                    ipc::ImgSend {
                        img: image::RgbaImage::from_pixel(dim.0, dim.1, image::Rgba(*color))
                            .to_vec()
                            .into_boxed_slice(),
                        path: color_path.clone(),
                        dim,
                        format: pixel_format,
                    },
                    namespace,
                    use_cache,
                    resize,
                    crop_gravity_str,
                    filter,
                    outputs,
                    None,
                );
            }
        }
        CliImage::Path(img_path) => {
            #[cfg(feature = "jxl")]
            jxl_oxide::integration::register_image_decoding_hook();

            let path = match img_path.canonicalize() {
                Ok(p) => p.display().to_string(),
                Err(e) => {
                    if let Some("-") = img_path.to_str() {
                        "STDIN".to_string()
                    } else {
                        return Err(format!("failed no canonicalize image path: {e}"));
                    }
                }
            };
            cache_path = path.clone();

            let is_video = {
                #[cfg(feature = "video")]
                {
                    video::is_video(img_path)
                }
                #[cfg(not(feature = "video"))]
                {
                    false
                }
            };

            if is_video {
                #[cfg(feature = "video")]
                {
                    let decoded = video::decode(img_path)?;
                    for (&dim, outputs) in dims.iter().zip(outputs) {
                        let (first_frame, animation) = video::compress_to_animation(
                            &decoded,
                            dim,
                            img.resize,
                            img.filter,
                            img.fill_color,
                            pixel_format,
                        )?;
                        img_req_builder.push(
                            ipc::ImgSend {
                                img: first_frame,
                                path: path.clone(),
                                dim,
                                format: pixel_format,
                            },
                            namespace,
                            use_cache,
                            resize,
                            crop_gravity_str,
                            filter,
                            outputs,
                            Some(animation),
                        );
                    }
                }
            } else {
                let imgbuf = ImgBuf::new(img_path)?;
                match imgbuf.decode_prepare() {
                    DecodeBuffer::RasterImage(imgbuf) => {
                        let img_raw = imgbuf.decode(pixel_format)?;

                        for (&dim, outputs) in dims.iter().zip(outputs) {
                            let animation = if imgbuf.is_animated() {
                                match cache::load_animation_frames(
                                    &path.clone(),
                                    dim,
                                    resize,
                                    pixel_format,
                                ) {
                                    Ok(Some(animation)) => Some(animation),
                                    otherwise => {
                                        if let Err(e) = otherwise {
                                            eprintln!(
                                                "Error loading cache for {}: {e}",
                                                img_path.display()
                                            );
                                        }
                                        Some({
                                            ipc::Animation {
                                                animation: compress_frames(
                                                    imgbuf.as_frames()?,
                                                    dim,
                                                    pixel_format,
                                                    make_filter(img.filter),
                                                    img.resize,
                                                    img.fill_color,
                                                )?
                                                .into_boxed_slice(),
                                            }
                                        })
                                    }
                                }
                            } else {
                                None
                            };

                            let blur = img.blur;
                            let dim_factor = img.dim;
                            let map_strength = img.map_strength;
                            let mut img = match img.resize {
                                ResizeStrategy::No => img_pad(&img_raw, dim, img.fill_color),
                                ResizeStrategy::Crop => img_resize_crop(
                                    &img_raw,
                                    dim,
                                    make_filter(img.filter),
                                    img.crop_gravity,
                                )?,
                                ResizeStrategy::Fit => img_resize_fit(
                                    &img_raw,
                                    dim,
                                    make_filter(img.filter),
                                    img.fill_color,
                                )?,
                                ResizeStrategy::Stretch => {
                                    img_resize_stretch(&img_raw, dim, make_filter(img.filter))?
                                }
                            };
                            effects::blur(&mut img, dim.0, dim.1, pixel_format.channels(), blur);
                            effects::dim(&mut img, pixel_format.channels(), dim_factor);
                            if let Some(stops) = &palette_stops {
                                effects::palette_map(
                                    &mut img,
                                    pixel_format.channels(),
                                    stops,
                                    map_strength,
                                );
                            }

                            img_req_builder.push(
                                ipc::ImgSend {
                                    img,
                                    path: path.clone(),
                                    dim,
                                    format: pixel_format,
                                },
                                namespace,
                                use_cache,
                                resize,
                                crop_gravity_str,
                                filter,
                                outputs,
                                animation,
                            );
                        }
                    }
                    // Vector images are different because we can render them at any scale. So we
                    // always make sure to render them at the largest possible scale without distortion
                    DecodeBuffer::VectorImage(imgbuf) => {
                        for (&dim, outputs) in dims.iter().zip(outputs) {
                            let filter = img.filter.as_str();
                            let img_raw = imgbuf.decode(pixel_format, dim.0, dim.1)?;
                            let blur = img.blur;
                            let dim_factor = img.dim;
                            let map_strength = img.map_strength;
                            let mut img = match img.resize {
                                ResizeStrategy::No => img_pad(&img_raw, dim, img.fill_color),
                                ResizeStrategy::Crop => img_resize_crop(
                                    &img_raw,
                                    dim,
                                    make_filter(img.filter),
                                    img.crop_gravity,
                                )?,
                                ResizeStrategy::Fit => img_resize_fit(
                                    &img_raw,
                                    dim,
                                    make_filter(img.filter),
                                    img.fill_color,
                                )?,
                                ResizeStrategy::Stretch => {
                                    img_resize_stretch(&img_raw, dim, make_filter(img.filter))?
                                }
                            };
                            effects::blur(&mut img, dim.0, dim.1, pixel_format.channels(), blur);
                            effects::dim(&mut img, pixel_format.channels(), dim_factor);
                            if let Some(stops) = &palette_stops {
                                effects::palette_map(
                                    &mut img,
                                    pixel_format.channels(),
                                    stops,
                                    map_strength,
                                );
                            }
                            img_req_builder.push(
                                ipc::ImgSend {
                                    img,
                                    path: path.clone(),
                                    dim,
                                    format: pixel_format,
                                },
                                namespace,
                                use_cache,
                                resize,
                                crop_gravity_str,
                                filter,
                                outputs,
                                None,
                            );
                        }
                    }
                }
            }
        }
    }

    if use_cache && update_cached_disconnected_outputs {
        img_req_builder.update_disconnected_caches(cache_path, namespace, outputs);
    }

    Ok(img_req_builder.build())
}

#[allow(clippy::type_complexity)]
fn get_format_dims_and_outputs(
    requested_outputs: &[String],
    namespace: &str,
) -> Result<(ipc::PixelFormat, Vec<(u32, u32)>, Vec<Vec<String>>), String> {
    let mut outputs: Vec<Vec<String>> = Vec::new();
    let mut dims: Vec<(u32, u32)> = Vec::new();
    let mut imgs: Vec<ipc::BgImg> = Vec::new();

    let socket = IpcSocket::client(namespace).map_err(|err| err.to_string())?;
    RequestSend::Query
        .send(&socket)
        .map_err(|e| e.to_string())?;
    let bytes = socket.recv().map_err(|err| err.to_string())?;
    drop(socket);
    let answer = Answer::receive(bytes);
    match answer {
        Answer::Info(infos) => {
            let mut format = ipc::PixelFormat::Argb;
            for info in &infos {
                format = info.pixel_format;
                let info_img = &info.img;
                let name = info.name.to_string();
                if !requested_outputs.is_empty() && !requested_outputs.contains(&name) {
                    continue;
                }
                let real_dim = info.real_dim();
                if let Some((_, output)) = dims
                    .iter_mut()
                    .zip(&imgs)
                    .zip(&mut outputs)
                    .find(|((dim, img), _)| real_dim == **dim && info_img == *img)
                {
                    output.push(name);
                } else {
                    outputs.push(vec![name]);
                    dims.push(real_dim);
                    imgs.push(info_img.clone());
                }
            }
            if outputs.is_empty() {
                Err("none of the requested outputs are valid".to_owned())
            } else {
                Ok((format, dims, outputs))
            }
        }
        _ => unreachable!(),
    }
}

fn split_cmdline_outputs(outputs: &str) -> Box<[String]> {
    outputs
        .split(',')
        .map(ToOwned::to_owned)
        .filter(|s| !s.is_empty())
        .collect()
}

fn restore_from_cache(requested_outputs: &[String], namespace: &str) -> Result<(), String> {
    let (_, _, outputs) = get_format_dims_and_outputs(requested_outputs, namespace)?;

    for output in outputs.iter().flatten() {
        if let Err(e) = restore_output(output, namespace) {
            eprintln!("WARNING: failed to load cache for output {output}: {e}");
        }
    }

    Ok(())
}

fn restore_output(output: &str, namespace: &str) -> Result<(), String> {
    let cache_data = common::cache::read_cache_file(output)
        .map_err(|e| format!("failed to read cache file: {e}"))?;
    let cache = match common::cache::get_previous_image_cache(output, namespace, &cache_data) {
        Ok(Some(cache)) => cache,
        Ok(None) => return Err("cache entry does not exist".to_string()),
        Err(e) => return Err(e.to_string()),
    };

    let crop_gravity = cache
        .crop_gravity
        .map(|v| CropGravity::from_str(v).unwrap_or_default());

    process_xwww_args(
        &Xwww::Img(cli::Img {
            all: false,
            image: cli::parse_image(cache.img_path)?,
            outputs: output.to_string(),
            namespace: vec![namespace.to_string()],
            no_cache: true,
            #[allow(deprecated)]
            no_resize: false,
            resize: ResizeStrategy::from_str(cache.resize).unwrap_or(ResizeStrategy::Crop),
            crop_gravity,
            fill_color: [0, 0, 0, 255],
            filter: Filter::from_str(cache.filter).unwrap_or(Filter::Lanczos3),
            blur: 0,
            dim: 1.0,
            map_palette: None,
            map_strength: 1.0,
            transition_type: cli::TransitionType::None,
            transition_step: std::num::NonZeroU8::MAX,
            transition_duration: 0.0,
            transition_fps: 30,
            transition_angle: 0.0,
            transition_pos: cli::CliPosition {
                x: cli::CliCoord::Pixel(0.0),
                y: cli::CliCoord::Pixel(0.0),
            },
            invert_y: false,
            transition_bezier: (0.0, 0.0, 0.0, 0.0),
            transition_wave: (0.0, 0.0),
        }),
        namespace,
    )?;
    Ok(())
}

#[cfg(feature = "scene")]
fn run_scene_cli(args: &cli::Scene) -> Result<(), String> {
    match &args.command {
        cli::SceneCommand::Check(check) => {
            scene::SceneEngine::check(&check.script)?;
            println!("{}: ok", check.script.display());
            Ok(())
        }
        cli::SceneCommand::Render(render) => render_scene_png(render),
        cli::SceneCommand::Run(run) => run_scene(run),
    }
}

#[cfg(feature = "scene")]
fn parse_scene_size(raw: &str) -> Result<(u32, u32), String> {
    let invalid = || format!("invalid size '{raw}', expected WxH (e.g. 2560x1440)");
    let (width, height) = raw.split_once(['x', 'X']).ok_or_else(invalid)?;
    let width: u32 = width.trim().parse().map_err(|_| invalid())?;
    let height: u32 = height.trim().parse().map_err(|_| invalid())?;
    if width == 0 || height == 0 {
        return Err(format!("invalid size '{raw}', dimensions must be positive"));
    }
    Ok((width, height))
}

#[cfg(feature = "scene")]
fn render_scene_png(render: &cli::SceneRender) -> Result<(), String> {
    let (width, height) = parse_scene_size(&render.size)?;
    let timeout = Duration::from_millis(render.timeout_ms.max(1));
    let mut engine = scene::SceneEngine::load_with_assets(
        &render.script,
        width,
        height,
        timeout,
        render.palette.as_deref(),
        &render.assets,
    )?;

    engine.render(0.0)?;

    let background = engine.palette().background;
    let rgb = engine
        .with_canvas(|canvas| canvas.to_flat(3, false, [background.r, background.g, background.b]));
    let image = image::RgbImage::from_raw(width, height, rgb.into_vec())
        .ok_or("failed to build the output image")?;
    image
        .save(&render.output)
        .map_err(|e| format!("failed to save {}: {e}", render.output.display()))?;

    println!("wrote {}", render.output.display());
    Ok(())
}

#[cfg(feature = "scene")]
fn run_scene(run: &cli::SceneRun) -> Result<(), String> {
    let namespace = run.namespace.first().map(String::as_str).unwrap_or("");
    let requested_outputs = split_cmdline_outputs(&run.outputs);
    let (format, dims, outputs) = get_format_dims_and_outputs(&requested_outputs, namespace)?;

    let timeout = Duration::from_millis(run.timeout_ms.max(1));
    let mut engines = Vec::with_capacity(dims.len());
    for (dim, output_group) in dims.iter().zip(&outputs) {
        let engine = scene::SceneEngine::load(&run.script, dim.0, dim.1, timeout, run.palette.as_deref())?
            .with_palette_fade(Duration::from_millis(run.palette_fade));
        engines.push((engine, *dim, output_group.clone()));
    }

    let path = format!(
        "scene:{}",
        run.script
            .canonicalize()
            .unwrap_or_else(|_| run.script.clone())
            .display()
    );
    let fps = run.fps.max(1);
    let interval = Duration::from_secs_f64(1.0 / f64::from(fps));
    let start = std::time::Instant::now();

    /* Entry transition: only the first frame uses it, and the loop waits for it to finish so
       the following instant frames do not cut it off. */
    let entry_transition = make_transition(&run.transition_args());
    let entry_wait = if matches!(
        entry_transition.transition_type,
        ipc::TransitionType::None
    ) {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(f64::from(run.transition_duration).max(0.0))
    };

    eprintln!(
        "xwww scene: {} output(s) at {fps} fps (Ctrl-C to stop)",
        engines.len()
    );

    let mut first_frame = vec![true; engines.len()];

    loop {
        let frame_start = std::time::Instant::now();
        let t = start.elapsed().as_secs_f64();
        let mut sent_first = false;

        for (index, (engine, dim, output_group)) in engines.iter_mut().enumerate() {
            if let Err(e) = engine.render(t) {
                eprintln!("xwww scene: {e}");
                continue;
            }
            if !first_frame[index] && !engine.take_dirty() {
                continue;
            }
            let transition = if first_frame[index] {
                sent_first = true;
                first_frame[index] = false;
                entry_transition.clone()
            } else {
                instant_transition()
            };
            let background = engine.palette().background;
            let bytes = engine.with_canvas(|canvas| {
                canvas.to_flat(
                    format.channels(),
                    format.must_swap_r_and_b_channels(),
                    [background.r, background.g, background.b],
                )
            });
            send_scene_frame(bytes, *dim, format, output_group, &path, namespace, transition)?;
        }

        let elapsed = frame_start.elapsed();
        let budget = if sent_first && entry_wait > Duration::ZERO {
            entry_wait
        } else {
            interval
        };
        if elapsed < budget {
            std::thread::sleep(budget - elapsed);
        }
    }
}

/// A step-255 transition: switches to the new frame immediately.
#[cfg(feature = "scene")]
fn instant_transition() -> ipc::Transition {
    ipc::Transition {
        transition_type: ipc::TransitionType::None,
        duration: 0.0,
        step: std::num::NonZeroU8::MAX,
        fps: 1,
        angle: 0.0,
        pos: ipc::Position::new(ipc::Coord::Percent(0.5), ipc::Coord::Percent(0.5)),
        bezier: (0.0, 0.0, 1.0, 1.0),
        wave: (0.0, 0.0),
        invert_y: false,
    }
}

#[cfg(feature = "scene")]
#[allow(clippy::too_many_arguments)]
fn send_scene_frame(
    bytes: Box<[u8]>,
    dim: (u32, u32),
    format: ipc::PixelFormat,
    outputs: &[String],
    path: &str,
    namespace: &str,
    transition: ipc::Transition,
) -> Result<(), String> {
    let mut builder = ipc::ImageRequestBuilder::new(transition)
        .map_err(|e| format!("failed to create the image request: {e}"))?;
    builder.push(
        ipc::ImgSend {
            path: path.to_string(),
            dim,
            format,
            img: bytes,
        },
        namespace,
        false,
        "crop",
        None,
        "Lanczos3",
        outputs,
        None,
    );

    let socket = IpcSocket::client(namespace).map_err(|e| e.to_string())?;
    RequestSend::Img(builder.build())
        .send(&socket)
        .map_err(|e| e.to_string())?;
    let answer = Answer::receive(socket.recv().map_err(|e| e.to_string())?);
    match answer {
        Answer::Ok => Ok(()),
        _ => Err("unexpected answer from the daemon".to_string()),
    }
}
