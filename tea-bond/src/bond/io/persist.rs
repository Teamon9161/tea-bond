use super::default_dir;
use crate::{SmallStr, bond::Bond};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

pub type BondMapType = HashMap<SmallStr, Arc<Bond>>;

/// 全量债券数据的内存映射，按需加载。
/// 通过 `BONDS_INFO_MAP` 控制序列化文件路径，未设置时使用 `~/.tea-bond/bonds_info.map`。
pub(crate) static BOND_MAP: LazyLock<Mutex<Option<BondMapType>>> =
    LazyLock::new(|| Mutex::new(None));

fn map_path() -> PathBuf {
    std::env::var_os("BONDS_INFO_MAP")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_dir().join("bonds_info.map"))
}

fn normalize_code(code: &str) -> SmallStr {
    if code.contains('.') {
        code.into()
    } else {
        format!("{code}.IB").into()
    }
}

fn load_from_disk(path: &Path) -> Result<HashMap<SmallStr, Arc<Bond>>> {
    let file = File::open(path).with_context(|| format!("Open bond map at {:?}", path))?;
    let mut reader = BufReader::new(file);
    let cfg = bincode::config::standard();
    bincode::serde::decode_from_std_read(&mut reader, cfg)
        .context("Deserialize bond map with bincode")
}

fn flush_to_disk(path: &Path, map: &HashMap<SmallStr, Arc<Bond>>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Create parent dir for bond map at {:?}", path))?;
    }
    let file = File::create(path).with_context(|| format!("Create bond map at {:?}", path))?;
    let mut writer = BufWriter::new(file);
    let cfg = bincode::config::standard();
    bincode::serde::encode_into_std_write(map, &mut writer, cfg).context("Serialize bond map")?;
    writer.flush().context("Flush bond map writer")
}

fn ensure_loaded() {
    let mut guard = BOND_MAP.lock();
    if guard.is_some() {
        return;
    }
    let path = map_path();
    match load_from_disk(&path) {
        Ok(map) => {
            *guard = Some(map);
        }
        Err(_err) => {
            *guard = Some(HashMap::new());
        }
    }
}

/// Clears the global bond cache (`BOND_MAP`), freeing all cached bonds.
#[inline]
pub fn free_bond_map() {
    let mut guard = BOND_MAP.lock();
    if let Some(s) = guard.as_mut() {
        s.clear();
    }
}

impl Bond {
    /// 从内存/磁盘映射读取债券；首次调用会尝试从磁盘加载。
    pub fn read_disk(code: &str) -> Result<Arc<Self>> {
        ensure_loaded();
        let normalized = normalize_code(code);
        let guard = BOND_MAP.lock();
        let map = guard.as_ref().expect("bond map should be initialized");
        map.get(normalized.as_str())
            .cloned()
            .with_context(|| format!("Bond {} not found in disk map", normalized))
    }

    /// 写入/更新内存映射；`flush_all` 为 true 时同步落盘。
    pub fn save_disk(&self, flush_all: bool) -> Result<()> {
        ensure_loaded();
        let mut guard = BOND_MAP.lock();
        let map = guard.as_mut().expect("bond map should be initialized");
        map.insert(self.bond_code().into(), Arc::new(self.clone()));
        if flush_all {
            let path = map_path();
            flush_to_disk(&path, map)?;
        }
        Ok(())
    }
}
