#![deny(clippy::all)]

use std::collections::HashMap;

use float_pigment_css::{self, StyleSheetResource};
use napi::bindgen_prelude::{Buffer, Uint8Array};
use napi_derive::napi;
#[napi(object)]
pub struct CompileOption {
	pub output_type: String,
	pub tag_name_prefix: String,
	pub src: HashMap<String, Uint8Array>,
}
#[napi(object)]
pub struct CompileResultItem {
  pub content: Buffer,
  pub warnings: Vec<String>,
}
#[napi(object)]
pub struct CompileResult {
  pub import_index: Buffer,
  pub files: HashMap<String, CompileResultItem>,
}

#[napi]
pub fn compile_sync(cfg: CompileOption) -> CompileResult {
  let output_type = cfg.output_type;
  let mut result = CompileResult { import_index: Buffer::from(Vec::new()), files: HashMap::new() };
  if output_type == "bincode" {
    let mut ssr = StyleSheetResource::new();
    
    // 处理tag名称前缀
    if !cfg.tag_name_prefix.is_empty() {
      for (name, _) in &cfg.src {
        ssr.add_tag_name_prefix(name.as_str(), &cfg.tag_name_prefix);
      }
    }
    
    for  (name, data) in &cfg.src {
      // First, check if the vector has data
      let content= String::from_utf8_lossy(data.as_ref()).into_owned();
      let str = content.as_str();
      let warn = ssr.add_source(name.as_str(), str);
      let mut arr: Vec<String> = Vec::new();
      for w in warn {
        arr.push(String::from(w.message.as_str()));
      }
      result.files.insert(name.clone(), CompileResultItem {
        content: Buffer::from(Vec::new()),
        warnings: arr,
      });
    }
    for  (name, _) in &cfg.src {
      if let Some(bincode) = ssr.serialize_bincode(name.as_str()) {
        if let Some(file) = result.files.get_mut(name) {
          file.content = Buffer::from(bincode);
        }
      }
    }
    let index = ssr.generate_import_indexes();
    result.import_index = Buffer::from(index.serialize_bincode());
  }
  result 
}

#[napi]
pub async fn compile(cfg: CompileOption) -> napi::Result<CompileResult> {
  // 将耗时操作放在线程池中执行
  napi::tokio::spawn(async move {
    let output_type = cfg.output_type;
    let mut result = CompileResult { import_index: Buffer::from(Vec::new()), files: HashMap::new() };
    if output_type == "bincode" {
      let mut ssr = StyleSheetResource::new();
      
      // 处理tag名称前缀
      if !cfg.tag_name_prefix.is_empty() {
        for (name, _) in &cfg.src {
          ssr.add_tag_name_prefix(name.as_str(), &cfg.tag_name_prefix);
        }
      }
      
      // 添加源文件
      for (name, data) in &cfg.src {
        let content = String::from_utf8_lossy(data.as_ref()).into_owned();
        let str = content.as_str();
        let warn = ssr.add_source(name.as_str(), str);
        let mut arr: Vec<String> = Vec::new();
        for w in warn {
          arr.push(String::from(w.message.as_str()));
        }
        result.files.insert(name.clone(), CompileResultItem {
          content: Buffer::from(Vec::new()),
          warnings: arr,
        });
      }
      
      // 序列化每个文件
      for (name, _) in &cfg.src {
        if let Some(bincode) = ssr.serialize_bincode(name.as_str()) {
          if let Some(file) = result.files.get_mut(name) {
            file.content = Buffer::from(bincode);
          }
        }
      }
      
      // 生成导入索引
      let index = ssr.generate_import_indexes();
      result.import_index = Buffer::from(index.serialize_bincode());
    }
    
    Ok(result)
  })
  .await
  .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Async task error: {}", e)))?
}
