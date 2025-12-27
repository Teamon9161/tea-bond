# Tea-Bond
[![PyPI](https://img.shields.io/pypi/v/tea-bond)](pybond)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Tea-Bond 是一个高性能的 Rust 债券计算库，专门用于中国债券市场的量化分析。提供完整的债券定价、收益率计算、期现套利分析等功能，同时支持 Python 绑定。

## 🚀 主要特性

### 债券计算功能
- **债券基本计算**: 收益率(YTM)、久期、应计利息、净价/全价转换
- **批量计算**: 高效的向量化计算支持

### 期货相关功能
- **转换因子计算**: 基于中金所标准的转换因子计算
- **交割日计算**: 自动计算期货合约的最后交易日和交割日
- **可交割券判断**: 判断债券是否符合期货交割标准

### 期现套利分析
- **基差计算**: 债券净价与期货价格的基差分析
- **净基差(BNOC)**: 扣除持有成本后的净基差
- **隐含回购利率(IRR)**: 期现套利的隐含融资成本
- **持有收益(Carry)**: 持有期间的收益分析

### 批量计算

* 支持通过Polars Expr计算相关指标
* 支持在Numba NoPython下使用


### 盈亏计算

- **实时盈亏**: 债券交易的实时盈亏计算
- **历史盈亏**: 基于历史交易数据的盈亏分析

## 📦 项目结构

```
tea-bond/
├── tea-bond/          # 核心债券计算库
├── tea-calendar/      # 交易日历库
├── pybond/           # Python 绑定库
```

## 🛠 安装

### Rust 依赖
```toml
[dependencies]
tea-bond = { git = "https://github.com/teamon9161/tea-bond.git", branch="master"}
```

### Python 安装
```bash
pip install tea-bond
```

## 📖 使用示例

### Python 示例

#### 基本债券计算

```python
from pybond import Bond

# 创建债券对象
bond = Bond(240215)
trade_date = "2025-02-03"
# 计算应计利息
accrued_interest = bond.accrued_interest(trade_date)
print(f"应计利息: {accrued_interest:.6f}")

# 根据收益率计算价格
ytm = 0.02115
clean_price = bond.clean_price(trade_date, ytm)
dirty_price = bond.dirty_price(trade_date, ytm)
print(f"净价: {clean_price:.6f}")
print(f"全价: {dirty_price:.6f}")
```

#### 期现套利分析

```python
from pybond import TfEvaluator
from datetime import date

# 创建期现套利评估器
evaluator = TfEvaluator(
    "T2409",
    bond,
    date=date(2024, 8, 12),
    future_price=102.10
    bond_ytm=0.02115,
    capital_rate=0.019
)

# 计算指标

print(f"转换因子: {evaluator.cf:.4f}")
print(f"基差: {evaluator.basis_spread:.6f}")
print(f"净基差: {evaluator.net_basis_spread:.6f}")
print(f"隐含回购利率: {evaluator.irr:.6f}")
print(f"持有收益: {evaluator.carry:.6f}")

```

### Rust 示例

#### 基本债券计算
```rust
use tea_bond::*;
use chrono::NaiveDate;

// 创建债券对象
let bond = Bond {
    bond_code: "240006.IB".into(),
    mkt: Market::IB,
    par_value: 100.0,
    cp_type: CouponType::CouponBear,
    interest_type: InterestType::Fixed,
    cp_rate: 0.0228,
    inst_freq: 1,
    carry_date: NaiveDate::from_ymd_opt(2024, 3, 25).unwrap(),
    maturity_date: NaiveDate::from_ymd_opt(2031, 3, 25).unwrap(),
    day_count: BondDayCount::ActAct,
    ..Default::default()
};

let trade_date = NaiveDate::from_ymd_opt(2024, 8, 12).unwrap();

// 计算应计利息
let accrued_interest = bond.calc_accrued_interest(trade_date).unwrap();
println!("应计利息: {:.6}", accrued_interest);

// 根据收益率计算价格
let ytm = 0.02115;
let clean_price = bond.calc_clean_price_with_ytm(trade_date, ytm).unwrap();
let dirty_price = bond.calc_dirty_price_with_ytm(trade_date, ytm).unwrap();
println!("净价: {:.6}", clean_price);
println!("全价: {:.6}", dirty_price);

// 计算久期
let duration = bond.calc_duration(trade_date, ytm).unwrap();
println!("修正久期: {:.6}", duration);
```

#### 期现套利分析
```rust
use tea_bond::*;
use chrono::NaiveDate;

// 创建期现套利评估器
let mut evaluator = TfEvaluator {
    date: NaiveDate::from_ymd_opt(2024, 8, 12).unwrap(),
    future: FuturePrice {
        future: Future::new("T2409").into(),
        price: 105.5,
    },
    bond: BondYtm {
        bond: bond.into(),
        ytm: 0.02115,
    },
    capital_rate: 0.019,
    ..Default::default()
};

// 计算所有指标
let result = evaluator.calc_all().unwrap();

println!("转换因子: {:.4}", result.cf.unwrap());
println!("基差: {:.6}", result.basis_spread.unwrap());
println!("净基差: {:.6}", result.net_basis_spread.unwrap());
println!("隐含回购利率: {:.6}", result.irr.unwrap());
println!("持有收益: {:.6}", result.carry.unwrap());
```

## 📊 支持的计算指标

### 债券基本指标
- **收益率(YTM)**: 到期收益率计算
- **久期**: 修正久期和麦考利久期
- **应计利息**: 基于不同计息规则的应计利息
- **净价/全价**: 债券净价和全价的相互转换

### 期现套利指标
- **基差(Basis)**: `债券净价 - 期货价格 × 转换因子`
- **持有收益(Carry)**: `(交割日应计利息 - 交易日应计利息 + 期间付息) + 资金成本`
- **净基差(BNOC)**: `基差 - 持有收益`
- **隐含回购利率(IRR)**: `(发票价格 + 期间付息 - 债券全价) / (债券全价 × 剩余天数/365 - 加权平均期间付息)`
- **期现利差**: `发票价格 - 债券全价 + 期间付息`

### 转换因子计算
基于中金所标准的转换因子计算公式：
```
CF = (c/f + c/r + (1-c/r)/(1+r/f)^(n-1)) / (1+r/f)^(x*f/12) - (1-x*f/12)*c/f
```
其中：
- `r`: 虚拟券票面利率(3%)
- `c`: 可交割债券票面利率  
- `f`: 年付息次数
- `n`: 剩余付息次数
- `x`: 交割月到下一付息月的月份数

## 🔧 功能特性

### 高性能计算
- 使用 Rust 编写，提供卓越的计算性能
- 支持批量计算和向量化操作
- 内存安全和零成本抽象

### 完整的日历支持
- 上交所、中金所(CFFEX)交易日历
- 银行间市场交易日历
- 自动处理节假日和调休

### Python 集成
- 提供完整的 Python 绑定
- 支持 NumPy 和 Pandas 数据结构
- 支持Numba nopython模式
- 易于集成到现有的 Python 量化框架

## 🔍 API 文档

### 主要类型

#### Bond
债券基本信息和计算功能
```rust
pub struct Bond {
    pub bond_code: SmallStr,
    pub mkt: Market,
    pub par_value: f64,
    pub cp_type: CouponType,
    pub interest_type: InterestType,
    pub cp_rate: f64,
    pub inst_freq: i32,
    pub carry_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub day_count: BondDayCount,
    // ...
}
```

#### Future
期货合约信息
```rust
pub struct Future {
    pub code: SmallStr,
    pub market: Option<SmallStr>,
}
```

#### TfEvaluator
期现套利评估器
```rust
pub struct TfEvaluator {
    pub date: NaiveDate,
    pub future: FuturePrice,
    pub bond: BondYtm,
    pub capital_rate: f64,
    pub reinvest_rate: Option<f64>,
    // ...
}
```

## 🧪 测试

运行测试：
```bash
# Rust 测试
cargo test

# Python 测试
cd pybond && python -m pytest
```

## 📋 系统要求

- Rust 1.70+ (对于 Rust 开发)
- Python 3.8+ (对于 Python 绑定)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目使用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- 感谢中金所提供的转换因子计算公式
- 参考文献：徐亮《国债期货投资策略与实务》

---

**注意**: 本库仅用于学术研究和教育目的，不构成投资建议。使用本库进行实际交易的风险由用户自行承担。