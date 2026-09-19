# Phase3 45 examples
9 original features x 3 (SAT/UNSAT/UNKNOWN) = 27
+ 7 gap syntaxes x 2 (SAT/UNSAT) + extra combined = 18
Total 45

## Original 9
- struct, enum, vec, loop, match, mod, async, io, unsafe, lifetime (9 x 3)

## New gap syntaxes (Phase3+)
- pat_or: FullPat::Or (a|b|c) — parse_pat.rs 280 行
- pat_range: FullPat::Range (0..10, 0..=10, ..10, 0..) — parse_pat.rs
- closure: FullExpr::Closure (||, |x|, |x,y|, move ||) — parse_expr.rs 640 行
- return: FullExpr::Return (return expr)
- break: FullExpr::Break { label, expr } (break 'label 42)
- try: FullExpr::Try (x?)
- cast: FullExpr::Cast (x as i32, x as *mut i32)
- range_expr: FullExpr::Range (0..10, 0..=10)

All implemented zero third-party deps, handwritten recursive descent, integrated with HandwrittenParser coverage and oracle::compare_parsers CI.
Each has SAT/UNSAT verified via constraints_v2 and pipeline_v2.

## Oracle fix
- New endpoint POST /api/v2/fix_oracle — auto writes syn FullProgram back to core AST (ProgramV2) and generates v2 Poly
- UI button "Oracle 差異一鍵修復" calls fix_oracle and shows diff
- CI: test_oracle_gap fails if missing_in_handwritten non-empty for critical gaps, prints OracleReport
