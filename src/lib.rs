use imara_diff::intern::InternedInput;
use imara_diff::{diff, Algorithm};
use serde::Serialize;
use std::ops::Range;
use wasm_bindgen::prelude::*;

// ── 恐慌时在浏览器控制台打印 ──────────────────────────────
extern crate console_error_panic_hook;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ══════════════════════════════════════════════════════════
// 序列化结构体
// ══════════════════════════════════════════════════════════

/// 一条 diff 操作，序列化为前端消费的 JSON
#[derive(Serialize)]
pub struct Op {
    /// "equal" | "delete" | "insert"
    #[serde(rename = "type")]
    pub kind:         &'static str,
    pub before_start: u32,
    pub before_end:   u32,
    pub after_start:  u32,
    pub after_end:    u32,
}

// ══════════════════════════════════════════════════════════
// 内部：将 imara-diff 的原始 hunks 展开为含 equal 段的完整序列
// ══════════════════════════════════════════════════════════

/// imara-diff 只回调「变动」hunk（before_range, after_range）；
/// 本函数将间隙填充为 equal 操作，返回完整操作序列。
fn expand_ops(
    hunks:   &[(Range<u32>, Range<u32>)],
    total_b: u32,
    total_a: u32,
) -> Vec<Op> {
    let mut ops: Vec<Op> = Vec::with_capacity(hunks.len() * 3);
    let mut cur_b: u32 = 0;
    let mut cur_a: u32 = 0;

    for (br, ar) in hunks {
        // equal 段：[cur_b, br.start) × [cur_a, ar.start)
        if cur_b < br.start {
            ops.push(Op {
                kind:         "equal",
                before_start: cur_b,
                before_end:   br.start,
                after_start:  cur_a,
                after_end:    ar.start,
            });
        }

        // delete + insert 合并为一条 replace（同时有删除和插入时）
        // 分开发送会让前端 buildSlots 把 del 和 ins 配对成 chg，
        // 必须保证 delete 紧跟 insert，中间不能插入 equal。
        if br.start < br.end && ar.start < ar.end {
            // replace：先 del 再 ins，before/after 侧各自记录真实区间
            ops.push(Op {
                kind:         "delete",
                before_start: br.start,
                before_end:   br.end,
                after_start:  ar.start,   // ins 紧跟，after 侧起点一致
                after_end:    ar.start,
            });
            ops.push(Op {
                kind:         "insert",
                before_start: br.end,     // before 侧零宽占位
                before_end:   br.end,
                after_start:  ar.start,
                after_end:    ar.end,
            });
        } else if br.start < br.end {
            // 纯删除
            ops.push(Op {
                kind:         "delete",
                before_start: br.start,
                before_end:   br.end,
                after_start:  ar.start,
                after_end:    ar.start,
            });
        } else if ar.start < ar.end {
            // 纯插入
            ops.push(Op {
                kind:         "insert",
                before_start: br.start,   // before 侧零宽占位
                before_end:   br.start,
                after_start:  ar.start,
                after_end:    ar.end,
            });
        }

        cur_b = br.end;
        cur_a = ar.end;
    }

    // 末尾 equal 段
    if cur_b < total_b {
        ops.push(Op {
            kind:         "equal",
            before_start: cur_b,
            before_end:   total_b,
            after_start:  cur_a,
            after_end:    total_a,
        });
    }

    ops
}

// ══════════════════════════════════════════════════════════
// 公开 WASM 接口
// ══════════════════════════════════════════════════════════

/// 对两段多行文本做行级 diff，返回 JSON 字符串。
///
/// # 参数
/// - `before`    原始文本（整块，含换行符）
/// - `after`     修改后文本
/// - `algorithm` `"histogram"`（默认，推荐）或 `"myers"`
///
/// # 返回值
/// JSON 数组，每个元素：
/// ```json
/// { "type": "equal"|"delete"|"insert",
///   "before_start": u32, "before_end": u32,
///   "after_start":  u32, "after_end":  u32 }
/// ```
/// 行号从 0 开始，区间为左闭右开 `[start, end)`。
#[wasm_bindgen]
pub fn diff_lines(before: &str, after: &str, algorithm: &str) -> String {
    let alg = match algorithm {
        "myers" => Algorithm::Myers,
        _       => Algorithm::Histogram, // 默认 histogram
    };

    // imara-diff 的行数统计（按 '\n' 分割，与前端 split('\n') 一致）
    let total_b = before.lines().count() as u32;
    let total_a = after.lines().count()  as u32;

    // InternedInput 会对每行做字符串驻留，大幅提升 diff 速度
    let input = InternedInput::new(before, after);

    let mut hunks: Vec<(Range<u32>, Range<u32>)> = Vec::new();
    diff(alg, &input, |b: Range<u32>, a: Range<u32>| {
        hunks.push((b, a));
    });

    let ops = expand_ops(&hunks, total_b, total_a);

    // serde_json 序列化；不应失败，unwrap 安全
    serde_json::to_string(&ops).unwrap_or_else(|e| {
        format!(r#"[{{"type":"error","msg":"{}"}}]"#, e)
    })
}

// ══════════════════════════════════════════════════════════
// 辅助：暴露版本字符串，方便前端验证 WASM 是否正确加载
// ══════════════════════════════════════════════════════════
#[wasm_bindgen]
pub fn wasm_version() -> String {
    format!(
        "large-text-diff-wasm v{} (imara-diff, {})",
        env!("CARGO_PKG_VERSION"),
        chrono::Utc::now().format("%Y-%m-%d"),
    )
}

// ══════════════════════════════════════════════════════════
// 单元测试（cargo test，非 WASM 环境运行）
// ══════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn ops(before: &str, after: &str) -> Vec<Op> {
        let total_b = before.lines().count() as u32;
        let total_a = after.lines().count()  as u32;
        let input = InternedInput::new(before, after);
        let mut hunks = Vec::new();
        diff(Algorithm::Histogram, &input, |b: Range<u32>, a: Range<u32>| {
            hunks.push((b, a));
        });
        expand_ops(&hunks, total_b, total_a)
    }

    #[test]
    fn identical_files() {
        let text = "line1\nline2\nline3\n";
        let result = ops(text, text);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "equal");
        assert_eq!(result[0].before_end, 3);
    }

    #[test]
    fn pure_insertion() {
        let before = "a\nb\n";
        let after  = "a\nx\nb\n";
        let result = ops(before, after);
        let kinds: Vec<_> = result.iter().map(|o| o.kind).collect();
        assert!(kinds.contains(&"insert"), "应包含 insert: {:?}", kinds);
        assert!(!kinds.contains(&"delete"), "不应包含 delete");
    }

    #[test]
    fn pure_deletion() {
        let before = "a\nb\nc\n";
        let after  = "a\nc\n";
        let result = ops(before, after);
        let kinds: Vec<_> = result.iter().map(|o| o.kind).collect();
        assert!(kinds.contains(&"delete"), "应包含 delete: {:?}", kinds);
    }

    #[test]
    fn modification() {
        let before = "hello\nworld\n";
        let after  = "hello\nrust\n";
        let result = ops(before, after);
        let kinds: Vec<_> = result.iter().map(|o| o.kind).collect();
        assert!(kinds.contains(&"delete") && kinds.contains(&"insert"),
            "修改行应同时包含 delete + insert: {:?}", kinds);
    }

    #[test]
    fn coverage_is_complete() {
        // 验证所有 before 行都被覆盖（equal + delete 的区间应覆盖 [0, total_b)）
        let before = "a\nb\nc\nd\ne\n";
        let after  = "a\nX\nc\nY\ne\n";
        let total_b = before.lines().count() as u32;
        let result = ops(before, after);

        let mut covered = vec![false; total_b as usize];
        for op in &result {
            if op.kind == "equal" || op.kind == "delete" {
                for i in op.before_start..op.before_end {
                    covered[i as usize] = true;
                }
            }
        }
        assert!(covered.iter().all(|&v| v), "存在未覆盖的 before 行: {:?}", covered);
    }

    #[test]
    fn json_is_valid() {
        let before = "foo\nbar\n";
        let after  = "foo\nbaz\n";
        // 直接调用公开函数（非 wasm 环境）
        let json = {
            let total_b = before.lines().count() as u32;
            let total_a = after.lines().count()  as u32;
            let input = InternedInput::new(before, after);
            let mut hunks = Vec::new();
            diff(Algorithm::Histogram, &input, |b: Range<u32>, a: Range<u32>| {
                hunks.push((b, a));
            });
            let ops = expand_ops(&hunks, total_b, total_a);
            serde_json::to_string(&ops).unwrap()
        };
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON 解析失败");
        assert!(parsed.is_array());
    }
}