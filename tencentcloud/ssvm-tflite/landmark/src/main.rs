use std::io::{self, Read};
use ssvm_tensorflow_interface;
use serde::Deserialize;

fn main() {
    let model_data: &[u8] = include_bytes!("lite-model_on_device_vision_classifier_landmarks_classifier_asia_V1_1.tflite");
    let labels = include_str!("probability-labels-en.txt");

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Error reading from STDIN");
    let obj: FaasInput = serde_json::from_str(&buffer).unwrap();
    // println!("{} {}", &(obj.body)[..5], obj.body.len());
    let img_buf = base64::decode_config(&(obj.body), base64::STANDARD).unwrap();
    // println!("Image buf size is {}", img_buf.len());

    let flat_img = ssvm_tensorflow_interface::load_jpg_image_to_rgb8(&img_buf, 321, 321);

    let mut session = ssvm_tensorflow_interface::Session::new(&model_data, ssvm_tensorflow_interface::ModelType::TensorFlowLite);
    session.add_input("uint8_image_input", &flat_img, &[1, 321, 321, 3])
           .run();
    let res_vec: Vec<f32> = session.get_output("transpose_1");

    let mut i = 0;
    let mut max_index: i32 = -1;
    let mut max_value: f32 = 0.0;
    while i < res_vec.len() {
        let cur = res_vec[i];
        if cur > max_value {
            max_value = cur;
            max_index = i as i32;
        }
        i += 1;
    }

    let mut confidence_zh = "可能有";
    let mut confidence_en = "could be";
    if max_value > 0.8 {
        confidence_zh = "非常可能有";
        confidence_en = "is very likely";
    } else if max_value > 0.5 {
        confidence_zh = "很可能有";
        confidence_en = "is likely";
    } else if max_value > 0.3 {
        confidence_zh = "可能有";
        confidence_en = "could be";
    }

    let mut label_lines = labels.lines();
    for _i in 0..max_index {
      label_lines.next();
    }

    let class_name = label_lines.next().unwrap().to_string();
    if max_value > 0.3 {
      println!("It {} a <a href='https://www.google.com/search?q={}'>{}</a> in the picture.", confidence_en.to_string(), class_name, class_name);
      println!("上传的图片里面{} <a href='https://www.baidu.com/s?wd={}'>{}</a>。", confidence_zh.to_string(), class_name, class_name);
    } else {
      println!("I could not detect an Asian landmark in the picture.");
      println!("上传的图片里面没有见到亚洲著名地标。");
    }
}

#[derive(Deserialize, Debug)]
struct FaasInput {
    body: String
}
