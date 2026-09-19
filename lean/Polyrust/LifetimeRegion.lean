-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
Phase3 — Lifetime Region 與 outlives 圖
對應 Rust：core/src/minirust/lifetime.rs, borrowck.rs
-/

namespace Polyrust
-- Lifetime 標識
inductive Lifetime : Type where
| named : String → Lifetime
| static : Lifetime
deriving DecidableEq, Repr

def Lifetime.isStatic : Lifetime → Bool
| .static => true
| _ => false

def Lifetime.isValid : Lifetime → Bool
| .named s => s.length > 0
| .static => true

-- Outlives 關係
structure Outlives where
  longer : Lifetime
  shorter : Lifetime
deriving DecidableEq, Repr

-- 簡化的圖表示：邊列表
structure LifetimeGraph where
  lifetimes : List Lifetime
  edges : List Outlives
deriving Repr

def LifetimeGraph.empty : LifetimeGraph := { lifetimes := [], edges := [] }

def LifetimeGraph.addLifetime (g : LifetimeGraph) (lt : Lifetime) : LifetimeGraph :=
  if g.lifetimes.contains lt then g else { g with lifetimes := lt :: g.lifetimes }

def LifetimeGraph.addOutlives (g : LifetimeGraph) (o : Outlives) : LifetimeGraph :=
  let g1 := g.addLifetime o.longer
  let g2 := g1.addLifetime o.shorter
  if g2.edges.contains o then g2 else { g2 with edges := o :: g2.edges }

def LifetimeGraph.hasSelfLoop : LifetimeGraph → Bool
| { edges := es, .. } => es.any (fun e => e.longer == e.shorter)

def LifetimeGraph.hasTwoCycle : LifetimeGraph → Bool
| { edges := es, .. } =>
  es.any (fun e1 =>
    es.any (fun e2 =>
      e1.longer == e2.shorter && e1.shorter == e2.longer && e1.longer != e1.shorter))

def LifetimeGraph.transitiveStep (g : LifetimeGraph) : LifetimeGraph :=
  let newEdges := g.edges.foldl (fun acc e1 =>
    let fromE1 := g.edges.filter (fun e2 => e2.longer == e1.shorter)
    fromE1.foldl (fun acc2 e2 =>
      let trans : Outlives := { longer := e1.longer, shorter := e2.shorter }
      if acc2.contains trans then acc2 else trans :: acc2
    ) acc
  ) g.edges
  { g with edges := newEdges }

def LifetimeGraph.outlivesHolds (g : LifetimeGraph) (longer shorter : Lifetime) : Bool :=
  if longer == shorter then true
  else if longer == .static then true
  else
    let direct := g.edges.any (fun e => e.longer == longer && e.shorter == shorter)
    if direct then true
    else
      g.edges.any (fun e1 =>
        e1.longer == longer &&
        g.edges.any (fun e2 => e2.longer == e1.shorter && e2.shorter == shorter))

-- Region：NLL 區間
structure Region where
  lifetime : Lifetime
  start : Nat
  fin : Nat
  borrowNode : Nat
deriving DecidableEq, Repr

def Region.overlaps (a b : Region) : Bool :=
  a.start < b.fin && b.start < a.fin

def checkNLL (regions : List Region) (g : LifetimeGraph) : List (Nat × Nat) :=
  regions.foldl (fun acc r1 =>
    regions.foldl (fun acc2 r2 =>
      if r1.borrowNode < r2.borrowNode && r1.overlaps r2 then
        if g.outlivesHolds r1.lifetime r2.lifetime || g.outlivesHolds r2.lifetime r1.lifetime then
          acc2
        else
          (r1.borrowNode, r2.borrowNode) :: acc2
      else acc2
    ) acc
  ) []

theorem static_outlives_all (g : LifetimeGraph) (lt : Lifetime) :
  g.outlivesHolds .static lt = true := by
  simp [LifetimeGraph.outlivesHolds]

theorem outlives_refl (g : LifetimeGraph) (lt : Lifetime) :
  g.outlivesHolds lt lt = true := by
  simp [LifetimeGraph.outlivesHolds]

def testGraph : LifetimeGraph :=
  LifetimeGraph.empty
    |>.addOutlives { longer := .named "a", shorter := .named "b" }
    |>.addOutlives { longer := .named "b", shorter := .named "c" }

def testGraphHolds : Bool := testGraph.outlivesHolds (.named "a") (.named "c")
def testGraphSelfLoop : Bool := testGraph.hasSelfLoop
def testGraphTwoCycle : Bool := testGraph.hasTwoCycle

end Polyrust
