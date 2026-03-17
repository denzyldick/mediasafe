use crate::database::{Database, Face};
use crate::emit_log;
use base64::Engine;
use ndarray::{Array2, Array4};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

// Conditional imports for AI Engines
#[cfg(not(target_os = "android"))]
use ort::{session::builder::GraphOptimizationLevel, session::Session};

#[cfg(target_os = "android")]
use tract_onnx::prelude::*;

pub struct MlContext {
    pub tx: UnboundedSender<String>,
    pub pending_count: Arc<AtomicUsize>,
    pub abort: Arc<std::sync::atomic::AtomicBool>,
}

// Model wrappers to handle different engine types
#[derive(Clone)]
enum ModelEngine {
    #[cfg(not(target_os = "android"))]
    Ort(Arc<Mutex<Session>>),
    #[cfg(target_os = "android")]
    Tract(Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, TypedModel>>),
}

impl ModelEngine {
    fn run(&self, input: Array4<f32>, _input_name: &str) -> Result<Vec<f32>, String> {
        match self {
            #[cfg(not(target_os = "android"))]
            ModelEngine::Ort(session) => {
                let shape = input.shape().to_vec();
                let data = input.into_raw_vec_and_offset().0;
                let tensor =
                    ort::value::Value::from_array((shape, data)).map_err(|e| e.to_string())?;
                let mut lock = session.lock().unwrap();
                let outputs = lock
                    .run(ort::inputs![_input_name => &tensor])
                    .map_err(|e| e.to_string())?;
                let mut results = Vec::new();
                for i in 0..outputs.len() {
                    if let Ok((_shape, data)) = outputs[i].try_extract_tensor::<f32>() {
                        results.extend_from_slice(data);
                    }
                }
                Ok(results)
            }
            #[cfg(target_os = "android")]
            ModelEngine::Tract(plan) => {
                let tract_tensor: tract_onnx::prelude::Tensor = input.into();
                let mut inputs = vec![];
                for _ in 0..plan.model().input_outlets().unwrap().len() {
                    inputs.push(tract_tensor.clone().into());
                }
                let result = plan.run(inputs.into()).map_err(|e| e.to_string())?;
                let mut results = Vec::new();
                for i in 0..result.len() {
                    if let Some(output) = result[i].as_slice::<f32>().ok() {
                        results.extend_from_slice(output);
                    }
                }
                Ok(results)
            }
        }
    }
}

fn compute_text_embeddings(
    #[cfg(not(target_os = "android"))] text_model: &mut Session,
    #[cfg(target_os = "android")] text_model: &SimplePlan<TypedFact, Box<dyn TypedOp>, TypedModel>,
    tokenizer: &tokenizers::Tokenizer,
) -> Vec<(String, Vec<f32>)> {
    let search_vocabulary = vec![
        "a passport",
        "a driver's license",
        "an id card",
        "a document",
        "a receipt",
        "a screenshot",
        "a meme",
        "a text message",
        "a cat",
        "a dog",
        "a pet",
        "an animal",
        "a car",
        "a vehicle",
        "a motorcycle",
        "a bicycle",
        "a person",
        "a selfie",
        "a group of people",
        "a crowd",
        "a building",
        "a house",
        "architecture",
        "a city",
        "a landscape",
        "nature",
        "a mountain",
        "a beach",
        "water",
        "food",
        "a meal",
        "a drink",
        "coffee",
        "a laptop",
        "a computer",
        "a phone",
        "a screen",
        "electronics",
        "a piece of furniture",
        "a room interior",
        "a sunset",
        "the sky",
        "clouds",
        "art",
        "a drawing",
        "a painting",
    ];

    let mut embeddings = Vec::new();
    for text_label in search_vocabulary {
        if let Ok(encoding) = tokenizer.encode(format!("a photo of {text_label}"), true) {
            #[cfg(not(target_os = "android"))]
            let mut ids = encoding
                .get_ids()
                .iter()
                .map(|&x| x as i64)
                .collect::<Vec<i64>>();
            #[cfg(target_os = "android")]
            let mut ids = encoding
                .get_ids()
                .iter()
                .map(|&x| x as i32)
                .collect::<Vec<i32>>();

            if ids.len() > 77 {
                ids.truncate(77);
            } else {
                while ids.len() < 77 {
                    ids.push(0);
                }
            }

            #[cfg(not(target_os = "android"))]
            let arr = Array2::from_shape_vec((1, 77), ids).unwrap();
            #[cfg(target_os = "android")]
            let arr = Array2::from_shape_vec((1, 77), ids).unwrap();

            #[cfg(not(target_os = "android"))]
            {
                let shape = arr.shape().to_vec();
                let data = arr.into_raw_vec_and_offset().0;
                if let Ok(id_tensor) = ort::value::Value::from_array((shape, data)) {
                    if let Ok(outputs) = text_model.run(ort::inputs!["input_ids" => &id_tensor]) {
                        if let Ok((_shape, text_emb_tensor)) =
                            outputs[0].try_extract_tensor::<f32>()
                        {
                            let mut text_embedding = vec![0.0; 512];
                            text_embedding.copy_from_slice(text_emb_tensor);
                            let text_norm: f32 =
                                text_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                            if text_norm > 0.0 {
                                for v in text_embedding.iter_mut() {
                                    *v /= text_norm;
                                }
                            }
                            embeddings.push((text_label.to_string(), text_embedding));
                        }
                    }
                }
            }

            #[cfg(target_os = "android")]
            {
                let tract_tensor: tract_onnx::prelude::Tensor = arr.into();
                // Pass BOTH input_ids and attention_mask (both same shape/type)
                // Many CLIP models require the mask to resolve the Range op internally
                let mut inputs = vec![];
                let input_count = text_model.model().input_outlets().unwrap().len();
                for _ in 0..input_count {
                    inputs.push(tract_tensor.clone().into());
                }

                if let Ok(result) = text_model.run(inputs.into()) {
                    if let Some(output) = result[0].as_slice::<f32>().ok() {
                        let mut text_embedding = output.to_vec();
                        // Handle models that return [1, sequence, 512] by taking the first token (CLS/BOS)
                        if text_embedding.len() > 512 {
                            text_embedding.truncate(512);
                        }
                        let text_norm: f32 =
                            text_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                        if text_norm > 0.0 {
                            for v in text_embedding.iter_mut() {
                                *v /= text_norm;
                            }
                        }
                        embeddings.push((text_label.to_string(), text_embedding));
                    }
                }
            }
        }
    }
    embeddings
}

pub fn start_background_worker(
    app: &AppHandle,
    config_path: String,
) -> (
    UnboundedSender<String>,
    Arc<AtomicUsize>,
    Arc<std::sync::atomic::AtomicBool>,
) {
    let (tx, mut rx) = unbounded_channel::<String>();
    let pending_count = Arc::new(AtomicUsize::new(0));
    let pending_count_clone = Arc::clone(&pending_count);
    let abort = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let abort_clone = Arc::clone(&abort);
    let app_handle = app.clone();
    let db_path = config_path.clone();
    let tx_for_thread = tx.clone();

    std::thread::spawn(move || {
        let models_dir = format!("{db_path}/models");
        let faces_dir = format!("{db_path}/faces");
        let _ = fs::create_dir_all(&models_dir);
        let _ = fs::create_dir_all(&faces_dir);

        let clip_visual_path = Path::new(&models_dir).join("clip-vit-base-patch32-visual.onnx");
        let clip_text_path = Path::new(&models_dir).join("clip-vit-base-patch32-text.onnx");
        let clip_tokenizer_path = Path::new(&models_dir).join("tokenizer.json");
        let ultraface_path = Path::new(&models_dir).join("version-RFB-320.onnx");

        let mut clip_visual: Option<ModelEngine> = None;
        let mut face_detector: Option<ModelEngine> = None;
        let mut tokenizer: Option<Arc<tokenizers::Tokenizer>>;
        let mut text_embeddings: Arc<Vec<(String, Vec<f32>)>> = Arc::new(Vec::new());
        let known_people: Arc<Mutex<Vec<(String, Vec<f32>)>>> = Arc::new(Mutex::new(Vec::new()));
        let mut engine_initialized = false;

        let db = Arc::new(Mutex::new(Database::new(&db_path)));
        let config = db.lock().unwrap().get_state();
        let num_threads: usize = config
            .get("scan_threads")
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        while let Some(photo_id) = rx.blocking_recv() {
            if photo_id == "__STATUS__" {
                let count = pending_count_clone.load(Ordering::SeqCst);
                let _ = app_handle.emit("indexing-progress", count);
                continue;
            }

            if photo_id == "__ABORT__" {
                abort_clone.store(true, Ordering::SeqCst);
                pending_count_clone.store(0, Ordering::SeqCst);
                // Clear the channel
                while rx.try_recv().is_ok() {}
                let _ = app_handle.emit("indexing-progress", 0);
                continue;
            }

            if !engine_initialized || photo_id == "__RELOAD_MODELS__" {
                abort_clone.store(false, Ordering::SeqCst);
                emit_log(
                    &app_handle,
                    "ML Worker: Initializing AI Engine...".to_string(),
                );

                #[cfg(not(target_os = "android"))]
                {
                    if !engine_initialized {
                        let _ = ort::init().with_name("siegu").commit();
                    }
                }
                engine_initialized = true;

                let is_ok = |p: &Path| {
                    p.exists() && p.metadata().map(|m| m.len()).unwrap_or(0) > 1024 * 1024
                };

                tokenizer = match tokenizers::Tokenizer::from_file(&clip_tokenizer_path) {
                    Ok(t) => Some(Arc::new(t)),
                    Err(_) => None,
                };

                if is_ok(&ultraface_path) {
                    emit_log(
                        &app_handle,
                        "ML Worker: Loading Face Detector...".to_string(),
                    );
                    #[cfg(not(target_os = "android"))]
                    {
                        match Session::builder()
                            .unwrap()
                            .with_optimization_level(GraphOptimizationLevel::Disable)
                            .unwrap()
                            .commit_from_file(&ultraface_path)
                        {
                            Ok(s) => {
                                face_detector = Some(ModelEngine::Ort(Arc::new(Mutex::new(s))))
                            }
                            Err(e) => emit_log(
                                &app_handle,
                                format!("ERROR: Face Detector load failed: {e}"),
                            ),
                        }
                    }
                    #[cfg(target_os = "android")]
                    {
                        match tract_onnx::onnx().model_for_path(&ultraface_path) {
                            Ok(mut model) => {
                                for i in 0..model.input_outlets().unwrap().len() {
                                    model
                                        .set_input_fact(i, f32::fact(&[1, 3, 240, 320]).into())
                                        .unwrap();
                                }
                                match model
                                    .into_typed()
                                    .and_then(|m| m.into_optimized())
                                    .and_then(|m| m.into_runnable())
                                {
                                    Ok(plan) => {
                                        face_detector = Some(ModelEngine::Tract(Arc::new(plan)))
                                    }
                                    Err(e) => emit_log(
                                        &app_handle,
                                        format!("ERROR: Face Detector optimization failed: {e}"),
                                    ),
                                }
                            }
                            Err(e) => emit_log(
                                &app_handle,
                                format!("ERROR: Face Detector path failed: {e}"),
                            ),
                        }
                    }
                }

                if is_ok(&clip_visual_path) {
                    emit_log(&app_handle, "ML Worker: Loading CLIP Visual...".to_string());
                    #[cfg(not(target_os = "android"))]
                    {
                        match Session::builder()
                            .unwrap()
                            .with_optimization_level(GraphOptimizationLevel::Disable)
                            .unwrap()
                            .commit_from_file(&clip_visual_path)
                        {
                            Ok(s) => clip_visual = Some(ModelEngine::Ort(Arc::new(Mutex::new(s)))),
                            Err(e) => emit_log(
                                &app_handle,
                                format!("ERROR: CLIP Visual load failed: {e}"),
                            ),
                        }
                    }
                    #[cfg(target_os = "android")]
                    {
                        match tract_onnx::onnx().model_for_path(&clip_visual_path) {
                            Ok(mut model) => {
                                for i in 0..model.input_outlets().unwrap().len() {
                                    model
                                        .set_input_fact(i, f32::fact(&[1, 3, 224, 224]).into())
                                        .unwrap();
                                }
                                match model
                                    .into_typed()
                                    .and_then(|m| m.into_optimized())
                                    .and_then(|m| m.into_runnable())
                                {
                                    Ok(plan) => {
                                        clip_visual = Some(ModelEngine::Tract(Arc::new(plan)))
                                    }
                                    Err(e) => emit_log(
                                        &app_handle,
                                        format!("ERROR: CLIP Visual initialization failed: {e}"),
                                    ),
                                }
                            }
                            Err(e) => emit_log(
                                &app_handle,
                                format!("ERROR: CLIP Visual path failed: {e}"),
                            ),
                        }
                    }
                }

                if is_ok(&clip_text_path) && tokenizer.is_some() {
                    emit_log(&app_handle, "ML Worker: Loading CLIP Text...".to_string());
                    #[cfg(not(target_os = "android"))]
                    {
                        if let Ok(mut s) = Session::builder()
                            .unwrap()
                            .with_optimization_level(GraphOptimizationLevel::Disable)
                            .unwrap()
                            .commit_from_file(&clip_text_path)
                        {
                            text_embeddings = Arc::new(compute_text_embeddings(
                                &mut s,
                                tokenizer.as_ref().unwrap(),
                            ));
                        }
                    }
                    #[cfg(target_os = "android")]
                    {
                        match tract_onnx::onnx().model_for_path(&clip_text_path) {
                            Ok(mut model) => {
                                // Most CLIP models have 2 inputs: input_ids and attention_mask
                                // We MUST set facts for both to resolve internal range ops
                                let input_count = model.input_outlets().unwrap().len();
                                for i in 0..input_count {
                                    let _ = model.set_input_fact(i, i32::fact(&[1, 77]).into());
                                }

                                match model
                                    .into_typed()
                                    .and_then(|m| m.into_optimized())
                                    .and_then(|m| m.into_runnable())
                                {
                                    Ok(plan) => {
                                        text_embeddings = Arc::new(compute_text_embeddings(
                                            &plan,
                                            tokenizer.as_ref().unwrap(),
                                        ))
                                    }
                                    Err(e) => emit_log(
                                        &app_handle,
                                        format!("ERROR: CLIP Text initialization failed: {e}"),
                                    ),
                                }
                            }
                            Err(e) => {
                                emit_log(&app_handle, format!("ERROR: CLIP Text path failed: {e}"))
                            }
                        }
                    }
                }

                let people_vec = db.lock().unwrap().get_all_people_with_embeddings();
                if let Ok(mut lock) = known_people.lock() {
                    *lock = people_vec;
                }
                emit_log(&app_handle, "ML Worker: Engine Ready.".to_string());
                if photo_id == "__RELOAD_MODELS__" {
                    continue;
                }
            }

            if photo_id == "__START__" {
                emit_log(
                    &app_handle,
                    "ML Worker: Checking for unindexed photos...".to_string(),
                );
                let unindexed = {
                    let lock = db.lock().unwrap();
                    let mut ids = Vec::new();
                    if let Ok(mut stmt) = lock
                        .connection
                        .prepare("SELECT id FROM photo WHERE indexed < 2")
                    {
                        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                            for id in rows.flatten() {
                                ids.push(id);
                            }
                        }
                    }
                    ids
                };

                if !unindexed.is_empty() {
                    emit_log(
                        &app_handle,
                        format!(
                            "ML Worker: Found {} photos to catch up on.",
                            unindexed.len()
                        ),
                    );
                    for id in unindexed {
                        let _ = tx_for_thread.send(id);
                    }
                }
                continue;
            }

            // Fetch photo entry from database for processing
            let photo_entry = {
                let lock = db.lock().unwrap();
                let sql = "SELECT id, location, encoded, latitude, longitude, created, indexed FROM photo WHERE id = ?1";
                lock.connection
                    .query_row(sql, [&photo_id], |row| {
                        Ok(crate::database::Photo {
                            id: row.get(0)?,
                            location: row.get(1)?,
                            encoded: row.get(2)?,
                            created: row.get(5).unwrap_or_default(),
                            objects: std::collections::HashMap::new(),
                            properties: std::collections::HashMap::new(),
                            latitude: row.get(3).unwrap_or(0.0),
                            longitude: row.get(4).unwrap_or(0.0),
                            favorite: false,
                            indexed: row.get(6).unwrap_or(0),
                        })
                    })
                    .ok()
            };

            if let Some(photo_entry) = photo_entry {
                if photo_entry.indexed >= 2 && photo_id != "__REPROCESS__" {
                    continue;
                }

                let photo_id_task = photo_entry.id.clone();
                let photo_loc_actual = photo_entry.location.clone();
                let current_encoded = photo_entry.encoded.clone();

                let app_handle_task = app_handle.clone();
                let pending_count_task = Arc::clone(&pending_count_clone);
                let clip_visual_task = clip_visual.clone();
                let face_detector_task = face_detector.clone();
                let text_embeddings_task = text_embeddings.clone();
                let known_people_task = known_people.clone();
                let faces_dir_task = faces_dir.clone();
                let db_task = Arc::clone(&db);
                let abort_task = Arc::clone(&abort_clone);

                pool.spawn(move || {
                    if abort_task.load(Ordering::SeqCst) {
                        return;
                    }

                    // 1. Check if high-quality thumbnail is needed
                    if current_encoded.is_empty()
                        || !current_encoded.starts_with("data:image/jpeg;base64,")
                    {
                        let ext = Path::new(&photo_loc_actual)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let is_video =
                            ["mp4", "mkv", "mov", "avi", "webm"].contains(&ext.as_str());

                        let new_encoded = if is_video {
                            crate::file::generate_video_thumbnail(&photo_loc_actual)
                                .unwrap_or_default()
                        } else {
                            crate::file::generate_thumbnail_base64(&photo_loc_actual, 400)
                                .unwrap_or_default()
                        };

                        if !new_encoded.is_empty() {
                            let lock = db_task.lock().unwrap();
                            lock.update_photo_thumbnail(&photo_id_task, &new_encoded);
                            // CRITICAL: Notify UI that thumbnail is ready to fix "blank photos" issue
                            if let Ok(photo) = lock.connection.query_row(
                                "SELECT id, location, encoded, latitude, longitude, created, indexed FROM photo WHERE id = ?1",
                                [&photo_id_task],
                                |row| Ok(crate::database::Photo {
                                    id: row.get(0)?,
                                    location: row.get(1)?,
                                    encoded: row.get(2)?,
                                    created: row.get(5).unwrap_or_default(),
                                    objects: std::collections::HashMap::new(),
                                    properties: std::collections::HashMap::new(),
                                    latitude: row.get(3).unwrap_or(0.0),
                                    longitude: row.get(4).unwrap_or(0.0),
                                    favorite: false,
                                    indexed: row.get(6).unwrap_or(0),
                                })
                            ) {
                                let _ = app_handle_task.emit("photo-updated", photo);
                            }
                        }
                    }


                    // 2. Load Image for AI
                    let image_res = image::open(&photo_loc_actual);
                    if let Ok(dynamic_img) = image_res {
                        let img = dynamic_img.to_rgb8();

                        // CLIP Visual
                        if let Some(ref visual_model) = clip_visual_task {
                            let resized = image::imageops::resize(
                                &img,
                                224,
                                224,
                                image::imageops::FilterType::Triangle,
                            );
                            let mut input_img = Array4::<f32>::zeros((1, 3, 224, 224));
                            for (x, y, pixel) in resized.enumerate_pixels() {
                                input_img[[0, 0, y as usize, x as usize]] =
                                    (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
                                input_img[[0, 1, y as usize, x as usize]] =
                                    (pixel[1] as f32 / 255.0 - 0.4578275) / 0.2613026;
                                input_img[[0, 2, y as usize, x as usize]] =
                                    (pixel[2] as f32 / 255.0 - 0.40821073) / 0.2757771;
                            }

                            if let Ok(data) = visual_model.run(input_img, "pixel_values") {
                                let mut visual_embedding = data;
                                let visual_norm: f32 = visual_embedding
                                    .iter()
                                    .map(|x| x * x)
                                    .sum::<f32>()
                                    .sqrt();
                                if visual_norm > 0.0 {
                                    for v in visual_embedding.iter_mut() {
                                        *v /= visual_norm;
                                    }
                                }

                                let mut similarities = Vec::new();
                                for (text_label, text_embedding) in text_embeddings_task.iter() {
                                    let dot_product: f32 = visual_embedding
                                        .iter()
                                        .zip(text_embedding.iter())
                                        .map(|(a, b)| a * b)
                                        .sum();
                                    similarities.push((text_label, dot_product));
                                }
                                similarities.sort_by(|a, b| {
                                    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                                });

                                let lock = db_task.lock().unwrap();
                                for (class_name, score) in similarities.iter().take(5) {
                                    let _ = lock.connection.execute(
                                        "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                                        (&photo_id_task, class_name, &score.to_string()),
                                    );
                                }
                            }
                        }

                        // Face Detection
                        if let Some(ref face_model) = face_detector_task {
                            let (orig_w, orig_h) = (img.width() as f32, img.height() as f32);
                            let resized = image::imageops::resize(
                                &img,
                                320,
                                240,
                                image::imageops::FilterType::Triangle,
                            );
                            let mut input = Array4::<f32>::zeros((1, 3, 240, 320));
                            for (x, y, pixel) in resized.enumerate_pixels() {
                                input[[0, 0, y as usize, x as usize]] =
                                    (pixel[0] as f32 - 127.0) / 128.0;
                                input[[0, 1, y as usize, x as usize]] =
                                    (pixel[1] as f32 - 127.0) / 128.0;
                                input[[0, 2, y as usize, x as usize]] =
                                    (pixel[2] as f32 - 127.0) / 128.0;
                            }

                            if let Ok(data) = face_model.run(input, "input") {
                                if data.len() >= 4420 * 6 {
                                    let scores = &data[..4420 * 2];
                                    let boxes = &data[4420 * 2..];
                                    let anchors = crate::face_detector::generate_anchors();
                                    let mut proposals = Vec::new();
                                    for i in 0..anchors.len() {
                                        let score = scores[i * 2 + 1];
                                        if score > 0.6 {
                                            let loc = [
                                                boxes[i * 4],
                                                boxes[i * 4 + 1],
                                                boxes[i * 4 + 2],
                                                boxes[i * 4 + 3],
                                            ];
                                            let decoded =
                                                crate::face_detector::decode(&loc, &anchors[i]);
                                            proposals.push((decoded, score));
                                        }
                                    }
                                    let keep = crate::face_detector::nms(&mut proposals, 0.3);

                                    for &idx in &keep {
                                        let bbox = proposals[idx].0;
                                        let xmin = (bbox[0] * orig_w).max(0.0) as u32;
                                        let ymin = (bbox[1] * orig_h).max(0.0) as u32;
                                        let xmax = (bbox[2] * orig_w).min(orig_w) as u32;
                                        let ymax = (bbox[3] * orig_h).min(orig_h) as u32;
                                        if xmax > xmin && ymax > ymin {
                                            let (w, h) = (xmax - xmin, ymax - ymin);
                                            if w > 20 && h > 20 {
                                                let face_crop = image::imageops::crop_imm(
                                                    &img, xmin, ymin, w, h,
                                                )
                                                .to_image();
                                                let face_id = format!(
                                                    "{photo_id_task}_face_{xmin}_{ymin}"
                                                );
                                                let crop_path =
                                                    format!("{faces_dir_task}/{face_id}.jpg");
                                                if face_crop.save(&crop_path).is_ok() {
                                                    let mut face_embedding = Vec::new();
                                                    if let Some(ref visual_model) =
                                                        clip_visual_task
                                                    {
                                                        let face_resized =
                                                            image::imageops::resize(
                                                                &face_crop,
                                                                224,
                                                                224,
                                                                image::imageops::FilterType::Triangle,
                                                            );
                                                        let mut face_input =
                                                            Array4::<f32>::zeros((1, 3, 224, 224));
                                                        for (x, y, pixel) in
                                                            face_resized.enumerate_pixels()
                                                        {
                                                            face_input
                                                                [[0, 0, y as usize, x as usize]] =
                                                                (pixel[0] as f32 / 255.0
                                                                    - 0.48145466)
                                                                    / 0.26862954;
                                                            face_input
                                                                [[0, 1, y as usize, x as usize]] =
                                                                (pixel[1] as f32 / 255.0
                                                                    - 0.4578275)
                                                                    / 0.2613026;
                                                            face_input
                                                                [[0, 2, y as usize, x as usize]] =
                                                                (pixel[2] as f32 / 255.0
                                                                    - 0.40821073)
                                                                    / 0.2757771;
                                                        }
                                                        if let Ok(emb) = visual_model
                                                            .run(face_input, "pixel_values")
                                                        {
                                                            let mut e = emb;
                                                            let norm: f32 = e
                                                                .iter()
                                                                .map(|x| x * x)
                                                                .sum::<f32>()
                                                                .sqrt();
                                                            if norm > 0.0 {
                                                                for v in e.iter_mut() {
                                                                    *v /= norm;
                                                                }
                                                            }
                                                            face_embedding = e;
                                                        }
                                                    }

                                                    let mut assigned_person_id = None;
                                                    if !face_embedding.is_empty() {
                                                        if let Ok(mut lock) =
                                                            known_people_task.lock()
                                                        {
                                                            let mut highest_similarity = 0.0f32;
                                                            let mut best_match_id = None;
                                                            for (person_id, person_centroid) in
                                                                lock.iter()
                                                            {
                                                                let dot_product: f32 =
                                                                    face_embedding
                                                                        .iter()
                                                                        .zip(
                                                                            person_centroid
                                                                                .iter(),
                                                                        )
                                                                        .map(|(a, b)| a * b)
                                                                        .sum();
                                                                if dot_product
                                                                    > highest_similarity
                                                                {
                                                                    highest_similarity =
                                                                        dot_product;
                                                                    best_match_id =
                                                                        Some(person_id.clone());
                                                                }
                                                            }
                                                            if highest_similarity > 0.75 {
                                                                assigned_person_id =
                                                                    best_match_id;
                                                            } else {
                                                                let lock_db =
                                                                    db_task.lock().unwrap();
                                                                let new_id = lock_db
                                                                    .create_anonymous_person(
                                                                        &face_embedding,
                                                                    );
                                                                lock.push((
                                                                    new_id.clone(),
                                                                    face_embedding.clone(),
                                                                ));
                                                                assigned_person_id =
                                                                    Some(new_id);
                                                            }
                                                        }
                                                    } else {
                                                        let lock_db = db_task.lock().unwrap();
                                                        let new_id = lock_db
                                                            .create_anonymous_person(&[]);
                                                        assigned_person_id = Some(new_id);
                                                    }

                                                    let mut buffer =
                                                        std::io::Cursor::new(Vec::new());
                                                    let _ = face_crop.write_to(
                                                        &mut buffer,
                                                        image::ImageOutputFormat::Jpeg(80),
                                                    );
                                                    let encoded = format!(
                                                        "data:image/jpeg;base64,{}",
                                                        base64::engine::general_purpose::STANDARD
                                                            .encode(buffer.get_ref())
                                                    );
                                                    let lock = db_task.lock().unwrap();
                                                    lock.store_face(Face {
                                                        photo_id: photo_id_task.clone(),
                                                        face_id: face_id.clone(),
                                                        crop_path,
                                                        encoded,
                                                        embedding: face_embedding,
                                                        person_id: assigned_person_id,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Mark as FULLY INDEXED
                    {
                        let lock = db_task.lock().unwrap();
                        lock.update_photo_indexed(&photo_id_task, 2);
                        let _ = lock.connection.execute(
                            "UPDATE photo SET sync_needed = 1 WHERE id = ?1",
                            [&photo_id_task],
                        );
                    }

                    // Proactively notify peer with FULL AI data
                    if let Some(state) = app_handle_task.try_state::<crate::WebRtcState>() {
                        let mut tx_lock = state.sync_tx.blocking_lock();
                        if let Some(tx) = tx_lock.as_mut() {
                            let db = db_task.lock().unwrap();
                            if let Ok(info) = db.get_photo_sync_info_by_id(&photo_id_task) {
                                let _ = tx.send(crate::transport::SyncMessage::SyncFile {
                                    photo: info,
                                });
                            }
                        }
                    }

                    let current = pending_count_task.fetch_sub(1, Ordering::SeqCst);
                    let _ = app_handle_task.emit("indexing-progress", current.saturating_sub(1));
                });
            }
        }
    });
    (tx, pending_count, abort)
}
