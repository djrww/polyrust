/-
# 公理審計（`#print axioms`）

本檔案**不屬於** `lake build` 的預設目標（只建 `Polyrust`）；它專門用來做
「證明的可信度審計」：

* 若任何定理的證明用了 `sorry`，其公理清單會出現 `sorryAx`；
* 若用了未被聲明的公理，會出現 `axiom ...`；
* 純 Lean 核心邏輯的結論只會列出 `propext`、`Classical.choice`、`Quot.sound`
  （Lean 標準三公理，與 Mathlib 相同）。

執行方式：`bash scripts/lean-audit.sh`（或 `cd lean && lake env lean Audit.lean`）。
-/

import Polyrust

open Polyrust

/-! ## 基礎層 -/

#print axioms Polyrust.monoMul_comm
#print axioms Polyrust.dividesM_antisymm
#print axioms Polyrust.ofBits_toBits
#print axioms Polyrust.squarefree_count

/-! ## T3(a) 子句–多項式對偶 -/

#print axioms Polyrust.field_poly_bit
#print axioms Polyrust.clause_duality
#print axioms Polyrust.cnf_duality

/-! ## T3(b) 消解代數與 CDCL 學習子句 -/

#print axioms Polyrust.resolution_identity
#print axioms Polyrust.resolvent_sat
#print axioms Polyrust.entails_resolvent
#print axioms Polyrust.learned_preserves_models
#print axioms Polyrust.learned_preserves_polyZero
#print axioms Polyrust.clausePoly_tautology
#print axioms Polyrust.unsat_iff_no_polyZero

/-! ## T4 Buchberger 終止性（含標準單項式計數） -/

#print axioms Polyrust.standard_implies_squarefree
#print axioms Polyrust.sublist_chain_length
#print axioms Polyrust.extension_bound
#print axioms Polyrust.buchberger_extension_bound
#print axioms Polyrust.no_infinite_sublist_chain

/-! ## T5 S-多項式準則 -/

#print axioms Polyrust.sPoly_mem_genIdeal
#print axioms Polyrust.genIdeal_insert_sPoly
#print axioms Polyrust.coprime_criterion
#print axioms Polyrust.sPoly_chain_decomposition
#print axioms Polyrust.chain_criterion
#print axioms Polyrust.sPoly_self

/-! ## T7(a) 約化 Gröbner 基唯一性 -/

#print axioms Polyrust.reduced_unique
#print axioms Polyrust.reduced_zero_of_mem
#print axioms Polyrust.lead_determines_element
#print axioms Polyrust.reduced_unique_univariate

/-! ## T6 Gröbner 判定定理 -/

#print axioms Polyrust.borrow_one_mem_no_root
#print axioms Polyrust.inIdeal_no_root
#print axioms Polyrust.interpolation
#print axioms Polyrust.no_root_certificate
#print axioms Polyrust.no_root_poly_certificate

/-! ## T1/T2/T6/T7 微實例 -/

#print axioms Polyrust.T1_micro
#print axioms Polyrust.T6_micro_unsat
#print axioms Polyrust.T6_micro_sat
#print axioms Polyrust.T7_arm_sat
#print axioms Polyrust.T7_wrong_arm_unsat

/-! ## L0 𝔽_p 嵌入保真 -/

#print axioms Polyrust.L0_mod_faithful
#print axioms Polyrust.eval_abs_bound
#print axioms Polyrust.L0_eval_faithful

/-! ## T8 QAP 忠實性 -/

#print axioms Polyrust.UniPoly.eval_mul
#print axioms Polyrust.UniPoly.div_linear
#print axioms Polyrust.UniPoly.vanishing_prod_dvd
#print axioms Polyrust.UniPoly.qap_duality

/-! ## 借用與所有權（T9 借用側） -/

#print axioms Polyrust.borrow_sat_iff_clean
#print axioms Polyrust.borrow_unsat_of_clash
#print axioms Polyrust.borrow_unsat_of_assign
#print axioms Polyrust.clashClause_duality
#print axioms Polyrust.assignClause_duality
#print axioms Polyrust.borrow_one_mem_no_root
#print axioms Polyrust.borrow_clash_one_mem
#print axioms Polyrust.borrow_assign_one_mem
#print axioms Polyrust.borrow_clash_no_root
#print axioms Polyrust.use_after_move_unsat
#print axioms Polyrust.p5_unsat
#print axioms Polyrust.p6_sat
#print axioms Polyrust.p5_p6_differ
#print axioms Polyrust.t9_borrow_decision

/-! ## T7(b) 宏展開（模板語法、同態、需求表回推） -/

#print axioms Polyrust.subst_id
#print axioms Polyrust.subst_comp
#print axioms Polyrust.expand_comp
#print axioms Polyrust.mem_vars_subst
#print axioms Polyrust.checkCtx_det
#print axioms Polyrust.checkCtx_expand
#print axioms Polyrust.check_expand_reflect
#print axioms Polyrust.check_expand_demands
#print axioms Polyrust.right_arm_typable
#print axioms Polyrust.expand_arm_forces_i32
#print axioms Polyrust.wrong_arm_untypable
#print axioms Polyrust.wrong_arm_no_root

/-! ## T9 端到端：判定等價與代碼生成 round-trip -/

#print axioms Polyrust.check_exclusive
#print axioms Polyrust.genC_sound
#print axioms Polyrust.genC_complete
#print axioms Polyrust.root_implies_typable
#print axioms Polyrust.typable_iff_root
#print axioms Polyrust.untypable_iff_no_root
#print axioms Polyrust.isMonoAt_of_root
#print axioms Polyrust.gen_length
#print axioms Polyrust.parseFuel_gen
#print axioms Polyrust.parse_gen
#print axioms Polyrust.arm_gating
#print axioms Polyrust.arm_gating_root
