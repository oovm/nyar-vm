use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::atomic::{AtomicUsize},
};
// --- ast.rs ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Base(String),                // e.g., "Int", "Bool"
    Arrow(Rc<Type>, Rc<Type>),   // Function type T -> U
    Product(Rc<Type>, Rc<Type>), // Product type T * U
    Adt(String),                 // Named ADT like "List"
    Id(Rc<Term>, Rc<Term>),      // Equality Type Id(A, x, y) - HoTT inspired
    Prop,                        // Type of propositions
}

#[derive(Debug, Clone)]
pub enum Term {
    Var(String),
    Lambda(String, Rc<Type>, Rc<Term>), // λx:T. body
    App(Rc<Term>, Rc<Term>),            // f(x)
    Pair(Rc<Term>, Rc<Term>),           // (a, b)
    Proj1(Rc<Term>),                    // π₁ p
    Proj2(Rc<Term>),                    // π₂ p
    Constructor(String, Vec<Rc<Term>>), // ADT Constructor, e.g., Cons(h, t)
    True,                               // Proposition True
    False,                              // Proposition False
    Eq(Rc<Term>, Rc<Term>),             // Equality Proposition a = b
    // Quantifiers (simplified representation)
    ForAll(String, Rc<Type>, Rc<Term>), // ∀x:T. P(x) - P(x) is the body
    Exists(String, Rc<Type>, Rc<Term>), // ∃x:T. P(x) - P(x) is the body
    // For internal use or advanced features
    Type(Rc<Type>), // Representing a type itself as a term
    // Placeholder for proof terms if needed later
    ProofTerm(String), // e.g., "refl", "trans", etc.
}

// Implement Hash and PartialEq/Eq carefully if using Rc/RefCell
// For simplicity, we might rely on structural hashing/equality here,
// but a robust implementation needs care with cycles and sharing.
// Simple structural equality/hashing for this example:

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Term::Var(s1), Term::Var(s2)) => s1 == s2,
            (Term::Lambda(v1, t1, b1), Term::Lambda(v2, t2, b2)) => v1 == v2 && t1 == t2 && b1 == b2,
            (Term::App(f1, a1), Term::App(f2, a2)) => f1 == f2 && a1 == a2,
            (Term::Pair(x1, y1), Term::Pair(x2, y2)) => x1 == x2 && y1 == y2,
            (Term::Proj1(p1), Term::Proj1(p2)) => p1 == p2,
            (Term::Proj2(p1), Term::Proj2(p2)) => p1 == p2,
            (Term::Constructor(c1, a1), Term::Constructor(c2, a2)) => c1 == c2 && a1 == a2,
            (Term::True, Term::True) => true,
            (Term::False, Term::False) => true,
            (Term::Eq(l1, r1), Term::Eq(l2, r2)) => l1 == l2 && r1 == r2,
            (Term::ForAll(v1, t1, b1), Term::ForAll(v2, t2, b2)) => v1 == v2 && t1 == t2 && b1 == b2,
            (Term::Exists(v1, t1, b1), Term::Exists(v2, t2, b2)) => v1 == v2 && t1 == t2 && b1 == b2,
            (Term::Type(t1), Term::Type(t2)) => t1 == t2,
            (Term::ProofTerm(s1), Term::ProofTerm(s2)) => s1 == s2,
            _ => false,
        }
    }
}
impl Eq for Term {}

impl Hash for Term {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state); // Hash the enum variant
        match self {
            Term::Var(s) => s.hash(state),
            Term::Lambda(v, t, b) => {
                v.hash(state);
                t.hash(state);
                b.hash(state);
            }
            Term::App(f, a) => {
                f.hash(state);
                a.hash(state);
            }
            Term::Pair(x, y) => {
                x.hash(state);
                y.hash(state);
            }
            Term::Proj1(p) => p.hash(state),
            Term::Proj2(p) => p.hash(state),
            Term::Constructor(c, args) => {
                c.hash(state);
                args.hash(state);
            }
            Term::True | Term::False => {} // No extra data
            Term::Eq(l, r) => {
                l.hash(state);
                r.hash(state);
            }
            Term::ForAll(v, t, b) => {
                v.hash(state);
                t.hash(state);
                b.hash(state);
            }
            Term::Exists(v, t, b) => {
                v.hash(state);
                t.hash(state);
                b.hash(state);
            }
            Term::Type(t) => t.hash(state),
            Term::ProofTerm(s) => s.hash(state),
        }
    }
}

// --- egraph.rs ---

// Unique ID for EClasses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EClassId(usize);

// Represents the structure of a term within the EGraph
// Uses EClassIds instead of recursive Term structures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ENode {
    Var(String),
    App(EClassId, EClassId),
    Pair(EClassId, EClassId),
    Proj1(EClassId),
    Proj2(EClassId),
    Constructor(String, Vec<EClassId>),
    True,
    False,
    Eq(EClassId, EClassId),
    // Quantifiers need careful handling - often expanded or handled by rules
    // For simplicity, we might not represent them directly as canonical ENodes
    // Or represent them with bound vars replaced by special symbols/ids
    ForAll(String, Rc<Type>, EClassId), // Type info needed, body is EClassId
    Exists(String, Rc<Type>, EClassId), // Type info needed, body is EClassId
    // Symbol or Literal nodes if needed
    // Symbol(String),
    // Literal(ValueType),
}

// Data associated with an EClass
#[derive(Debug, Clone)]
pub struct EClassData {
    nodes: HashSet<Rc<ENode>>, // Nodes equivalent in this class
    parents: HashSet<(Rc<ENode>, EClassId)>, // (ParentNode, ParentEClassId) where this EClass is used
    // Optional: Type information, cost model, canonical representation, etc.
    // pub inferred_type: Option<Rc<Type>>,
}

// Simple UnionFind
#[derive(Debug, Clone)]
pub struct UnionFind {
    parents: Vec<EClassId>,
    ranks: Vec<u32>, // Optional: for union by rank/size
}

impl UnionFind {
    fn new() -> Self {
        UnionFind { parents: Vec::new(), ranks: Vec::new() }
    }

    fn make_set(&mut self) -> EClassId {
        let id = EClassId(self.parents.len());
        self.parents.push(id);
        self.ranks.push(0);
        id
    }

    fn find(&mut self, id: EClassId) -> EClassId {
        if self.parents[id.0] != id {
            self.parents[id.0] = self.find(self.parents[id.0]); // Path compression
        }
        self.parents[id.0]
    }

    fn union(&mut self, id1: EClassId, id2: EClassId) -> (EClassId, bool) {
        let root1 = self.find(id1);
        let root2 = self.find(id2);
        if root1 != root2 {
            // Union by rank (optional optimization)
            if self.ranks[root1.0] < self.ranks[root2.0] {
                self.parents[root1.0] = root2;
                return (root2, true);
            }
            else if self.ranks[root1.0] > self.ranks[root2.0] {
                self.parents[root2.0] = root1;
                return (root1, true);
            }
            else {
                self.parents[root2.0] = root1;
                self.ranks[root1.0] += 1;
                return (root1, true);
            }
            // Simpler union:
            // self.parents[root1.0] = root2;
            // (root2, true)
        }
        else {
            (root1, false) // Already in the same set
        }
    }
}

#[derive(Debug)]
pub struct EGraph {
    union_find: UnionFind,
    hashcons: HashMap<Rc<ENode>, EClassId>, // Maps canonical ENodes to EClassId
    classes: HashMap<EClassId, EClassData>, // Maps canonical EClassId to data
    worklist: VecDeque<EClassId>,           // EClasses needing processing
    memo: HashMap<Rc<Term>, EClassId>,      // Memoization for add_term
    // For Quantifiers / Unique variables
    next_var_id: AtomicUsize,
    // ADT definitions, Type information etc. could be stored here
    // adt_defs: HashMap<String, AdtDefinition>,
}

impl EGraph {
    pub fn new() -> Self {
        EGraph {
            union_find: UnionFind::new(),
            hashcons: HashMap::new(),
            classes: HashMap::new(),
            worklist: VecDeque::new(),
            memo: HashMap::new(),
            next_var_id: AtomicUsize::new(0),
        }
    }

    fn get_data_mut(&mut self, id: EClassId) -> Option<&mut EClassData> {
        let root = self.union_find.find(id);
        self.classes.get_mut(&root)
    }

    fn get_data(&mut self, id: EClassId) -> Option<&EClassData> {
        let root = self.union_find.find(id);
        self.classes.get(&root)
    }

    // Recursively add a term to the EGraph
    pub fn add_term(&mut self, term: &Rc<Term>) -> EClassId {
        if let Some(&id) = self.memo.get(term) {
            return self.union_find.find(id); // Return canonical id
        }

        let enode = match term.as_ref() {
            Term::Var(s) => ENode::Var(s.clone()),
            Term::App(f, a) => {
                let f_id = self.add_term(f);
                let a_id = self.add_term(a);
                ENode::App(f_id, a_id)
            }
            Term::Pair(x, y) => {
                let x_id = self.add_term(x);
                let y_id = self.add_term(y);
                ENode::Pair(x_id, y_id)
            }
            Term::Proj1(p) => ENode::Proj1(self.add_term(p)),
            Term::Proj2(p) => ENode::Proj2(self.add_term(p)),
            Term::Constructor(c, args) => {
                let arg_ids = args.iter().map(|arg| self.add_term(arg)).collect();
                ENode::Constructor(c.clone(), arg_ids)
            }
            Term::True => ENode::True,
            Term::False => ENode::False,
            Term::Eq(l, r) => {
                let l_id = self.add_term(l);
                let r_id = self.add_term(r);
                ENode::Eq(l_id, r_id)
            }
            // Quantifiers are tricky. Maybe add them initially but don't canonicalize?
            // Or handle them purely via rewrite rules.
            // Simplified: Add ENode but rely on rules for logic.
            Term::ForAll(v, t, b) => {
                let b_id = self.add_term(b); // Need context for bound var 'v'
                ENode::ForAll(v.clone(), Rc::clone(t), b_id) // Not ideal w/o proper var handling
            }
            Term::Exists(v, t, b) => {
                let b_id = self.add_term(b);
                ENode::Exists(v.clone(), Rc::clone(t), b_id) // Not ideal w/o proper var handling
            }
            // Ignore Type/ProofTerm for now, or handle appropriately
            Term::Type(_) => panic!("Cannot add Type directly as canonical ENode yet"),
            Term::ProofTerm(_) => panic!("Cannot add ProofTerm directly as canonical ENode yet"),
            Term::Lambda(_, _, _) => panic!("Cannot add Lambda directly as canonical ENode yet (needs HOAS or similar)"),
        };

        let enode_rc = Rc::new(enode); // Wrap in Rc for sharing in hashcons/EClassData

        // Canonicalize children EClassIds before hashcons lookup
        let canonical_enode = self.canonicalize_enode(&enode_rc);
        let canonical_enode_rc = Rc::new(canonical_enode);

        let id = *self.hashcons.entry(Rc::clone(&canonical_enode_rc)).or_insert_with(|| {
            let new_id = self.union_find.make_set();
            let initial_data = EClassData {
                nodes: HashSet::new(), // Add the node below
                parents: HashSet::new(),
            };
            // Add the non-canonical node version to the actual class data
            // Important: store the original node structure with non-canonical IDs
            // But hashcons uses the canonical version for lookup.
            let mut data = initial_data;
            data.nodes.insert(Rc::clone(&enode_rc)); // Store original node structure
            self.classes.insert(new_id, data);
            self.worklist.push_back(new_id); // New class needs processing
            new_id
        });

        // Even if node existed, add the original term mapping to the memo
        let canonical_id = self.union_find.find(id);
        self.memo.insert(Rc::clone(term), canonical_id);

        // Add the (potentially non-canonical) enode to the nodeset of its class
        // This handles cases where e.g. f(a) exists, and later f(b) is added where a=b.
        // We need both f(a) and f(b) ENodes associated with the same EClass.
        let class_data = self.classes.get_mut(&canonical_id).unwrap();
        if class_data.nodes.insert(enode_rc) {
            // If we inserted a new node variant into an existing class,
            // it might trigger congruence or rewrites.
            self.worklist.push_back(canonical_id);
        }

        canonical_id
    }

    // Replace EClassIds in an ENode with their canonical representatives
    fn canonicalize_enode(&mut self, enode: &ENode) -> ENode {
        match enode {
            ENode::App(f, a) => ENode::App(self.find(*f), self.find(*a)),
            ENode::Pair(x, y) => ENode::Pair(self.find(*x), self.find(*y)),
            ENode::Proj1(p) => ENode::Proj1(self.find(*p)),
            ENode::Proj2(p) => ENode::Proj2(self.find(*p)),
            ENode::Constructor(c, args) => {
                let canon_args = args.iter().map(|id| self.find(*id)).collect();
                ENode::Constructor(c.clone(), canon_args)
            }
            ENode::Eq(l, r) => ENode::Eq(self.find(*l), self.find(*r)),
            // Keep non-canonicalizable nodes as they are
            ENode::Var(_) | ENode::True | ENode::False | ENode::ForAll(..) | ENode::Exists(..) => enode.clone(),
        }
    }

    pub fn find(&mut self, id: EClassId) -> EClassId {
        self.union_find.find(id)
    }

    // Merge two EClasses
    pub fn union(&mut self, id1: EClassId, id2: EClassId) -> bool {
        let root1 = self.find(id1);
        let root2 = self.find(id2);

        if root1 == root2 {
            return false; // Already equivalent
        }

        let (new_root, merged) = self.union_find.union(root1, root2);
        if !merged {
            return false;
        } // Should not happen if roots were different

        let old_root = if new_root == root1 { root2 } else { root1 };

        // Merge data (nodes, parents)
        let old_data = self.classes.remove(&old_root).unwrap(); // Take ownership
        let new_data = self.classes.get_mut(&new_root).unwrap();

        // Add old nodes to new class. If any node is new to the class, add to worklist.
        for node in old_data.nodes {
            if new_data.nodes.insert(node) {
                self.worklist.push_back(new_root);
            }
        }
        // Merge parents. Add to worklist as parent structure changed.
        // Also, update the parent pointers in the *other* classes. (Done in rebuild)
        for parent_info in old_data.parents {
            if new_data.parents.insert(parent_info) {
                self.worklist.push_back(new_root);
            }
        }

        // Add the merged class (new_root) to the worklist unconditionally
        // because its equivalence class has changed.
        self.worklist.push_back(new_root);

        println!("Merged {} into {}", old_root.0, new_root.0);
        true
    }

    // Update parent pointers after merges (part of rebuild's repair step)
    fn repair(&mut self, class_id: EClassId) {
        // Should be called with canonical class_id
        assert_eq!(self.find(class_id), class_id);

        let mut new_parents = HashSet::new();
        let data = self.classes.get_mut(&class_id).unwrap(); // Get mutable borrow

        // Iterate over a clone to allow modification of `parents`
        let current_parents = data.parents.clone();
        data.parents.clear(); // Clear old parents before re-adding valid ones

        for (parent_enode, parent_eclass_id) in current_parents {
            let canonical_parent_eclass_id = self.find(parent_eclass_id);
            // Check if the parent *structure* (after canonicalizing its children)
            // now points to a different EClass than the one it's currently in.
            let canon_parent_enode = Rc::new(self.canonicalize_enode(&parent_enode));

            if let Some(existing_id) = self.hashcons.get(&canon_parent_enode) {
                let canonical_existing_id = self.find(*existing_id);
                if canonical_existing_id != canonical_parent_eclass_id {
                    // Congruence merge! The parent structure now resolves to an existing different class.
                    println!(
                        "Repair Congruence Merge: {:?} ({}) with existing ({})",
                        parent_enode, canonical_parent_eclass_id.0, canonical_existing_id.0
                    );
                    self.union(canonical_parent_eclass_id, canonical_existing_id);
                }
            }
            // Re-add the parent pointer to the (potentially new) canonical parent class
            // Need to get the parent's data mutably again after potential merge
            let canon_parent_id_after_union = self.find(parent_eclass_id); // Re-find after potential union
            if let Some(parent_data) = self.classes.get_mut(&canon_parent_id_after_union) {
                parent_data.parents.insert((parent_enode.clone(), class_id)); // Point to the child's canonical id
            }

            // Also update the child's (current class_id) parent list
            // Need to get data mut again
            if let Some(current_class_data) = self.classes.get_mut(&class_id) {
                current_class_data.parents.insert((parent_enode, canon_parent_id_after_union));
            }
            else {
                // This class might have been merged away during the congruence merge.
                // The parents will be handled when the new merged class is processed.
            }
        }
    }

    // The main saturation loop
    pub fn rebuild(&mut self, rules: &[RewriteRule]) {
        while let Some(class_id) = self.worklist.pop_front() {
            let canonical_id = self.find(class_id);
            // Repair parent pointers and trigger congruence closure merges
            self.repair(canonical_id);

            // Apply rewrite rules
            if let Some(data) = self.classes.get(&canonical_id) {
                // Iterate over nodes in the class (need Rc for pattern matching)
                let nodes_in_class: Vec<Rc<ENode>> = data.nodes.iter().cloned().collect();
                for rule in rules {
                    for node in &nodes_in_class {
                        // Attempt to match the rule's LHS pattern against the node
                        if let Some(subst) = rule.match_pattern(node, self, canonical_id) {
                            // Check condition if it exists
                            let condition_holds = match &rule.condition {
                                Some(cond_pattern) => {
                                    if let Some(cond_id) = self.query_pattern(cond_pattern, &subst) {
                                        // Check if the condition EClass is equivalent to True EClass
                                        let true_id = self.add_term(&Rc::new(Term::True));
                                        self.find(cond_id) == self.find(true_id)
                                    }
                                    else {
                                        false // Condition pattern didn't match anything
                                    }
                                }
                                None => true, // No condition
                            };

                            if condition_holds {
                                // Apply the rewrite RHS
                                let rhs_term = rule.apply_rhs(&subst);
                                let rhs_id = self.add_term(&rhs_term); // Add the result
                                let current_node_id = self.find(canonical_id); // Re-find, could have changed

                                // If rule was `l -> r`, union the EClass of the matched node (l)
                                // with the EClass of the result (r).
                                println!("Applied rule: {} -> {}", rule.name, format!("{:?}", rhs_term));
                                self.union(current_node_id, rhs_id);
                                // Important: Union might add more work to the worklist
                            }
                        }
                    }
                }
            }
        }
        println!("Rebuild finished. EGraph state: {:?}", self.classes);
    }

    // Helper to query if a pattern exists and get its EClassId
    // (Simplified: assumes pattern variables are bound in subst)
    fn query_pattern(&mut self, pattern: &Pattern, subst: &Substitution) -> Option<EClassId> {
        let term = pattern.instantiate(subst); // Build term from pattern+subst
        let id = self.add_term(&term); // Add/find term in egraph
        Some(id)
    }

    // Get all terms equivalent to a given term
    pub fn get_equivalences(&mut self, term: &Rc<Term>) -> HashSet<Rc<ENode>> {
        let id = self.add_term(term);
        let canonical_id = self.find(id);
        self.classes.get(&canonical_id).map_or_else(HashSet::new, |data| data.nodes.clone())
    }
}

// --- rewrite.rs ---

// Pattern Variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternVar(String);

// Patterns for matching (simplified AST subset)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Var(PatternVar),
    App(Box<Pattern>, Box<Pattern>),
    Pair(Box<Pattern>, Box<Pattern>),
    Proj1(Box<Pattern>),
    Proj2(Box<Pattern>),
    Constructor(String, Vec<Pattern>),
    True,
    False,
    Eq(Box<Pattern>, Box<Pattern>),
    // Concrete terms can also be patterns
    TermNode(Rc<Term>), // Match a specific concrete term structure
}

// Substitution map: PatternVar -> EClassId
pub type Substitution = HashMap<PatternVar, EClassId>;

// Rewrite Rule
#[derive(Clone)] // Need Clone if storing Rules directly
pub struct RewriteRule {
    pub name: String,
    pub lhs: Pattern,
    pub rhs: Pattern,               // Use Pattern for RHS too, for applying substitutions
    pub condition: Option<Pattern>, // Optional condition pattern
}

impl RewriteRule {
    // Match LHS pattern against an ENode within a specific EClass
    // Returns Some(Substitution) on success
    fn match_pattern(
        &self,
        enode: &ENode,        // The node we are trying to match against in the eclass
        egraph: &mut EGraph,  // Need mutable access for find operation
        _eclass_id: EClassId, // The ID of the class containing the enode
    ) -> Option<Substitution> {
        let mut subst = Substitution::new();
        if self.match_recursive(&self.lhs, enode, egraph, &mut subst) { Some(subst) } else { None }
    }

    fn match_recursive(&self, pattern: &Pattern, enode: &ENode, egraph: &mut EGraph, subst: &mut Substitution) -> bool {
        match (pattern, enode) {
            (Pattern::Var(pv), _) => {
                // We need the EClassId associated with the matched ENode.
                // This requires looking up the ENode's class or having it passed down.
                // Simplification: Assume we can find the EClassId for the *structure* 'enode' represents.
                // This part is tricky. A better approach involves matching patterns directly on EClasses.
                // Let's assume `enode` corresponds to a structure whose EClassId is known (passed in outer func).
                // This matching logic needs refinement for a real implementation.

                // Placeholder: Find EClassId for the structure represented by enode
                let enode_rc = Rc::new(enode.clone());
                let canon_enode = egraph.canonicalize_enode(&enode_rc);
                let canon_enode_rc = Rc::new(canon_enode);
                if let Some(id) = egraph.hashcons.get(&canon_enode_rc) {
                    let canonical_id = egraph.find(*id);
                    if let Some(existing_id) = subst.get(pv) {
                        // If variable already bound, check for consistency
                        egraph.find(*existing_id) == canonical_id
                    }
                    else {
                        // Bind variable
                        subst.insert(pv.clone(), canonical_id);
                        true
                    }
                }
                else {
                    false // Should not happen if enode came from the egraph
                }
            }
            (Pattern::App(pf, pa), ENode::App(ef, ea)) => {
                // Need to match sub-patterns against the EClasses pointed to by ef, ea
                self.match_eclass(pf, *ef, egraph, subst) && self.match_eclass(pa, *ea, egraph, subst)
            }
            (Pattern::Pair(p1, p2), ENode::Pair(e1, e2)) => {
                self.match_eclass(p1, *e1, egraph, subst) && self.match_eclass(p2, *e2, egraph, subst)
            }
            (Pattern::Proj1(p), ENode::Proj1(e)) => self.match_eclass(p, *e, egraph, subst),
            (Pattern::Proj2(p), ENode::Proj2(e)) => self.match_eclass(p, *e, egraph, subst),
            (Pattern::Constructor(c1, ps), ENode::Constructor(c2, es)) => {
                c1 == c2
                    && ps.len() == es.len()
                    && ps.iter().zip(es.iter()).all(|(p, e_id)| self.match_eclass(p, *e_id, egraph, subst))
            }
            (Pattern::True, ENode::True) => true,
            (Pattern::False, ENode::False) => true,
            (Pattern::Eq(pl, pr), ENode::Eq(el, er)) => {
                self.match_eclass(pl, *el, egraph, subst) && self.match_eclass(pr, *er, egraph, subst)
            }
            // TODO: Pattern::TermNode matching against ENode structure
            _ => false, // Pattern doesn't match ENode structure
        }
    }

    // Helper to match a pattern against any node within an EClass
    fn match_eclass(&self, pattern: &Pattern, eclass_id: EClassId, egraph: &mut EGraph, subst: &mut Substitution) -> bool {
        let canonical_id = egraph.find(eclass_id);
        if let Some(data) = egraph.classes.get(&canonical_id) {
            // Try matching against any node in the target EClass
            for node in data.nodes.iter() {
                // Create a temporary substitution state for this attempt
                let mut temp_subst = subst.clone();
                if self.match_recursive(pattern, node, egraph, &mut temp_subst) {
                    // Success! Update the original substitution
                    *subst = temp_subst;
                    return true;
                }
            }
        }
        false // No node in the EClass matched the pattern
    }

    // Instantiate the RHS pattern using the substitution map
    // Produces a concrete Term to be added back to the EGraph
    fn apply_rhs(&self, subst: &Substitution) -> Rc<Term> {
        self.instantiate_pattern(&self.rhs, subst)
    }

    // Helper to instantiate a Pattern into a Term
    fn instantiate_pattern(&self, pattern: &Pattern, subst: &Substitution) -> Rc<Term> {
        match pattern {
            Pattern::Var(pv) => {
                // Find the EClassId in substitution. We need a representative Term.
                // This is a simplification! We should ideally work with EClassIds directly
                // or have a way to extract a canonical Term from an EClass.
                // Placeholder: Create a dummy Var term representing the EClass.
                let id = subst.get(pv).expect("Unbound pattern variable in RHS");
                // Ideally: egraph.get_canonical_term(*id)
                Rc::new(Term::Var(format!("e{}", id.0))) // Very basic placeholder
            }
            Pattern::App(f, a) => {
                let f_term = self.instantiate_pattern(f, subst);
                let a_term = self.instantiate_pattern(a, subst);
                Rc::new(Term::App(f_term, a_term))
            }
            Pattern::Pair(p1, p2) => {
                let t1 = self.instantiate_pattern(p1, subst);
                let t2 = self.instantiate_pattern(p2, subst);
                Rc::new(Term::Pair(t1, t2))
            }
            Pattern::Proj1(p) => Rc::new(Term::Proj1(self.instantiate_pattern(p, subst))),
            Pattern::Proj2(p) => Rc::new(Term::Proj2(self.instantiate_pattern(p, subst))),
            Pattern::Constructor(c, args) => {
                let arg_terms = args.iter().map(|p| self.instantiate_pattern(p, subst)).collect();
                Rc::new(Term::Constructor(c.clone(), arg_terms))
            }
            Pattern::True => Rc::new(Term::True),
            Pattern::False => Rc::new(Term::False),
            Pattern::Eq(l, r) => {
                let l_term = self.instantiate_pattern(l, subst);
                let r_term = self.instantiate_pattern(r, subst);
                Rc::new(Term::Eq(l_term, r_term))
            }
            Pattern::TermNode(term) => Rc::clone(term), // Concrete term
        }
    }
}

// Extension trait for Pattern construction convenience
impl Pattern {
    fn var(name: &str) -> Pattern {
        Pattern::Var(PatternVar(name.to_string()))
    }
    fn app(f: Pattern, a: Pattern) -> Pattern {
        Pattern::App(Box::new(f), Box::new(a))
    }
    fn eq(l: Pattern, r: Pattern) -> Pattern {
        Pattern::Eq(Box::new(l), Box::new(r))
    }
    // ... other helpers ...
}

// --- prover.rs ---
pub struct Prover {
    egraph: EGraph,
    rules: Vec<RewriteRule>,
    axioms: Vec<Rc<Term>>,
    // Add type definitions, ADT info etc. here if needed
    // type_defs: ...
}

impl Prover {
    pub fn new() -> Self {
        Prover { egraph: EGraph::new(), rules: Vec::new(), axioms: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: RewriteRule) {
        self.rules.push(rule);
    }

    pub fn add_axiom(&mut self, axiom: Rc<Term>) {
        self.axioms.push(axiom.clone());
        // Add axiom to egraph immediately? Or only at prove time?
        // Adding immediately allows early saturation.
        let _ = self.egraph.add_term(&axiom);
        // Axioms often represent equalities or propositions assumed true.
        // If axiom is `A = B`, union them. If `P` (a proposition), union `P` with `True`.
        match axiom.as_ref() {
            Term::Eq(l, r) => {
                let l_id = self.egraph.add_term(l);
                let r_id = self.egraph.add_term(r);
                self.egraph.union(l_id, r_id);
            }
            // Assume other top-level axioms are propositions asserted to be true
            Term::Var(_) | Term::App(_, _) | Term::Constructor(_, _) | Term::ForAll(_, _, _) | Term::Exists(_, _, _) => {
                let axiom_id = self.egraph.add_term(&axiom);
                let true_id = self.egraph.add_term(&Rc::new(Term::True));
                self.egraph.union(axiom_id, true_id);
            }
            _ => {} // Ignore Lambdas, Pairs, etc. as top-level axioms for now
        }
    }

    // Run saturation
    pub fn saturate(&mut self) {
        println!("Starting saturation...");
        self.egraph.rebuild(&self.rules);
        println!("Saturation complete.");
    }

    // Check if goal is proven
    pub fn check_goal(&mut self, goal: &Rc<Term>) -> bool {
        println!("Checking goal: {:?}", goal);
        // Ensure goal is in the egraph
        let goal_id = self.egraph.add_term(goal);

        match goal.as_ref() {
            Term::Eq(l, r) => {
                let l_id = self.egraph.add_term(l);
                let r_id = self.egraph.add_term(r);
                let proven = self.egraph.find(l_id) == self.egraph.find(r_id);
                println!("Equality Goal: {:?} = {:?} -> Proven: {}", l, r, proven);
                proven
            }
            // For general propositions P, check if P = True in the egraph
            _ => {
                let true_id = self.egraph.add_term(&Rc::new(Term::True));
                let proven = self.egraph.find(goal_id) == self.egraph.find(true_id);
                println!("Proposition Goal: {:?} = True -> Proven: {}", goal, proven);
                proven
            }
        }
    }

    // Attempt proof by contradiction
    pub fn prove_by_contradiction(&mut self, goal: Rc<Term>) -> bool {
        println!("Attempting proof by contradiction for: {:?}", goal);
        // Assume Not(goal) is true. How to represent Not?
        // Simplification: Assume 'False' represents contradiction.
        // We need a way to express the negation of the goal.
        // If goal is `A = B`, assume `Not(A = B)`.
        // If goal is `P`, assume `Not(P)`.
        // We need rules like `P and Not(P) -> False`.
        // Let's define Not(P) as Eq(P, False) for simplicity here.

        let negation: Rc<Term>;
        match goal.as_ref() {
            Term::Eq(l, r) => {
                // Not (a=b) -> (a=b) = False
                negation = Rc::new(Term::Eq(Rc::new(Term::Eq(Rc::clone(l), Rc::clone(r))), Rc::new(Term::False)));
            }
            _ => {
                // Not P -> P = False
                negation = Rc::new(Term::Eq(Rc::clone(&goal), Rc::new(Term::False)));
            }
        }

        println!("Adding negation assumption: {:?}", negation);
        let neg_id = self.egraph.add_term(&negation);
        let true_id = self.egraph.add_term(&Rc::new(Term::True));
        // Assume the negation is true
        self.egraph.union(neg_id, true_id);

        // Add a basic rule for contradiction: P = True, P = False |- False = True
        let p_var = Pattern::Var(PatternVar("P".into()));
        let contradiction_rule = RewriteRule {
            name: "Contradiction".into(),
            // Condition: find P such that P=True and P=False are both true.
            // This is hard to express directly as a rewrite rule trigger.
            // Instead, we check the result: if True == False after saturation.
            // Let's add the components and let saturation run.
            lhs: p_var.clone(), // Doesn't matter much, check happens after rebuild
            rhs: p_var,         // Doesn't matter much
            condition: None,
        };
        // No, the check is simpler: run rebuild and see if True == False

        self.saturate(); // Run saturation with the negation added

        let final_true_id = self.egraph.add_term(&Rc::new(Term::True));
        let final_false_id = self.egraph.add_term(&Rc::new(Term::False));

        let contradiction_found = self.egraph.find(final_true_id) == self.egraph.find(final_false_id);

        if contradiction_found {
            println!("Contradiction found (True == False)! Original goal is proven.");
        }
        else {
            println!("No contradiction found. Proof by contradiction failed.");
            // Important: Need to backtrack the assumption (remove negation) if part of larger proof system.
            // For this standalone check, we just report the result.
        }
        contradiction_found
    }

    // Simple example of how quantifiers *might* be handled via rules:
    // Rule: If (ForAll x: T. P(x)) is true, and 'a' of type T exists, add P(a).
    // This requires type information in the EGraph, which we haven't fully added.

    // Rule: If P(w) is true and w has type T, add (Exists x: T. P(x)).
}

// --- main.rs (or lib.rs for tests) ---

// Helper function to create Rc<Term> easily
fn term(t: Term) -> Rc<Term> {
    Rc::new(t)
}
fn tvar(name: &str) -> Rc<Term> {
    term(Term::Var(name.to_string()))
}
fn tapp(f: Rc<Term>, a: Rc<Term>) -> Rc<Term> {
    term(Term::App(f, a))
}
fn teq(l: Rc<Term>, r: Rc<Term>) -> Rc<Term> {
    term(Term::Eq(l, r))
}
// Pattern helpers
fn pvar(name: &str) -> Pattern {
    Pattern::Var(PatternVar(name.to_string()))
}
fn papp(f: Pattern, a: Pattern) -> Pattern {
    Pattern::App(Box::new(f), Box::new(a))
}
fn peq(l: Pattern, r: Pattern) -> Pattern {
    Pattern::Eq(Box::new(l), Box::new(r))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_equality() {
        let mut prover = Prover::new();
        // Axioms: a = b, b = c
        let a = tvar("a");
        let b = tvar("b");
        let c = tvar("c");
        prover.add_axiom(teq(Rc::clone(&a), Rc::clone(&b)));
        prover.add_axiom(teq(Rc::clone(&b), Rc::clone(&c)));

        // Saturate
        prover.saturate();

        // Goal: a = c
        let goal = teq(Rc::clone(&a), Rc::clone(&c));
        assert!(prover.check_goal(&goal)); // Should be true
        println!("Equivalences for 'a': {:?}", prover.egraph.get_equivalences(&a));
    }

    #[test]
    fn test_function_congruence() {
        let mut prover = Prover::new();
        let f = tvar("f");
        let g = tvar("g");
        let x = tvar("x");
        let y = tvar("y");
        let fx = tapp(Rc::clone(&f), Rc::clone(&x));
        let gy = tapp(Rc::clone(&g), Rc::clone(&y));

        // Axioms: f = g, x = y
        prover.add_axiom(teq(Rc::clone(&f), Rc::clone(&g)));
        prover.add_axiom(teq(Rc::clone(&x), Rc::clone(&y)));

        // Add terms f(x) and g(y) to the graph *before* saturation
        // so congruence can find them.
        let _fx_id = prover.egraph.add_term(&fx);
        let _gy_id = prover.egraph.add_term(&gy);

        // Saturate
        prover.saturate();

        // Goal: f(x) = g(y)
        let goal = teq(fx, gy);
        assert!(prover.check_goal(&goal));
    }

    #[test]
    fn test_basic_rewrite() {
        let mut prover = Prover::new();

        // Rule: add(0, x) -> x
        let rule = RewriteRule {
            name: "add_zero_l".into(),
            lhs: Pattern::Constructor(
                "add".into(),
                vec![
                    Pattern::Constructor("zero".into(), vec![]), // Represent 0 as Constructor("zero", [])
                    pvar("x"),
                ],
            ),
            rhs: pvar("x"),
            condition: None,
        };
        prover.add_rule(rule);

        // Term: add(zero, 5)  (Represent 5 as Constructor("const", [num_5]))
        let zero = term(Term::Constructor("zero".into(), vec![]));
        let five = term(Term::Constructor("const".into(), vec![tvar("5")])); // Simplified const representation
        let add_expr = term(Term::Constructor("add".into(), vec![zero, Rc::clone(&five)]));

        // Add term and saturate
        let _ = prover.egraph.add_term(&add_expr);
        prover.saturate();

        // Goal: add(zero, 5) = 5
        let goal = teq(add_expr, five);
        assert!(prover.check_goal(&goal));
    }

    #[test]
    fn test_conditional_rewrite() {
        let mut prover = Prover::new();

        // Rule: is_zero(x) == True && mult(x, y) -> zero
        let x = pvar("x");
        let y = pvar("y");
        let rule = RewriteRule {
            name: "mult_zero".into(),
            lhs: Pattern::Constructor("mult".into(), vec![x.clone(), y.clone()]),
            rhs: Pattern::Constructor("zero".into(), vec![]), // zero value
            condition: Some(peq(
                // Condition: is_zero(x) = True
                Pattern::Constructor("is_zero".into(), vec![x.clone()]),
                Pattern::True,
            )),
        };
        prover.add_rule(rule);

        // Axiom: is_zero(zero) = True
        let zero = term(Term::Constructor("zero".into(), vec![]));
        let is_zero_app = term(Term::Constructor("is_zero".into(), vec![Rc::clone(&zero)]));
        let is_zero_axiom = teq(is_zero_app, term(Term::True));
        prover.add_axiom(is_zero_axiom);

        // Term: mult(zero, 7)
        let seven = term(Term::Constructor("const".into(), vec![tvar("7")]));
        let mult_expr = term(Term::Constructor("mult".into(), vec![Rc::clone(&zero), Rc::clone(&seven)]));

        // Add term and saturate
        let _ = prover.egraph.add_term(&mult_expr);
        prover.saturate();

        // Goal: mult(zero, 7) = zero
        let goal = teq(mult_expr, zero);
        assert!(prover.check_goal(&goal));
    }

    #[test]
    fn test_proof_by_contradiction() {
        let mut prover = Prover::new();

        // Axioms: P = Q, Q = False
        let p = tvar("P");
        let q = tvar("Q");
        prover.add_axiom(teq(Rc::clone(&p), Rc::clone(&q)));
        prover.add_axiom(teq(Rc::clone(&q), term(Term::False)));

        // Goal: P = False (which should be provable directly)
        let direct_goal = teq(Rc::clone(&p), term(Term::False));
        // Saturate first
        prover.saturate();
        assert!(prover.check_goal(&direct_goal));

        // Now try proving P = True using contradiction (this should fail proof)
        // let fail_goal = teq(Rc::clone(&p), term(Term::True));
        // This requires a cleaner setup for proof state... Skipping P=True for now.

        // Let's try a classic: Assume Not P, derive False, conclude P.
        // Axioms: NotP -> False (equivalent to (NotP = True) -> (False = True))
        // We need a better way to model implication or negation rules.

        // Simpler contradiction test: Add P=True and P=False, check for True=False
        let mut prover_contra = Prover::new();
        let p2 = tvar("P2");
        prover_contra.add_axiom(teq(Rc::clone(&p2), term(Term::True)));
        prover_contra.add_axiom(teq(Rc::clone(&p2), term(Term::False)));
        prover_contra.saturate();

        let true_term = term(Term::True);
        let false_term = term(Term::False);
        let contra_goal = teq(Rc::clone(&true_term), Rc::clone(&false_term));
        // Check if True == False in the graph
        let true_id = prover_contra.egraph.add_term(&true_term);
        let false_id = prover_contra.egraph.add_term(&false_term);
        assert_eq!(prover_contra.egraph.find(true_id), prover_contra.egraph.find(false_id));
        println!("Contradiction test (P=T, P=F) leads to True == False: {}", true);

        // Test the prover's contradiction function
        // Goal: P (where axioms imply P=True)
        let mut prover_pbc = Prover::new();
        let p3 = tvar("P3");
        // Axiom: P3 = True
        prover_pbc.add_axiom(teq(Rc::clone(&p3), term(Term::True)));
        prover_pbc.saturate(); // Process axioms

        // Goal to prove by contradiction: P3
        let p3_goal = Rc::clone(&p3);
        // The function will assume Not(P3) (i.e., P3 = False) and run saturation.
        // Since P3=True is already known, adding P3=False should make True=False.
        assert!(prover_pbc.prove_by_contradiction(p3_goal));
    }

    // --- ADT and Product Type Tests (Conceptual) ---
    // These tests would require defining ADTs and product types more formally
    // within the prover setup (e.g., via Type definitions and ensuring
    // constructors/projections are handled correctly).

    #[test]
    fn test_adt_equality() {
        let mut prover = Prover::new();
        // Assume List::Cons and List::Nil are constructors
        let one = term(Term::Constructor("Int".into(), vec![tvar("1")]));
        let two = term(Term::Constructor("Int".into(), vec![tvar("2")]));
        let nil = term(Term::Constructor("List::Nil".into(), vec![]));
        let list1 = term(Term::Constructor("List::Cons".into(), vec![Rc::clone(&one), Rc::clone(&nil)]));
        let list2 = term(Term::Constructor("List::Cons".into(), vec![Rc::clone(&one), Rc::clone(&nil)]));
        let list3 = term(Term::Constructor("List::Cons".into(), vec![Rc::clone(&two), Rc::clone(&nil)]));

        // Add terms
        let id1 = prover.egraph.add_term(&list1);
        let id2 = prover.egraph.add_term(&list2);
        let id3 = prover.egraph.add_term(&list3);

        prover.saturate();

        // Check: list1 = list2 ? (Should be true by structure)
        assert_eq!(prover.egraph.find(id1), prover.egraph.find(id2));
        // Check: list1 = list3 ? (Should be false)
        assert_ne!(prover.egraph.find(id1), prover.egraph.find(id3));

        // Check goal form
        let goal_eq = teq(list1.clone(), list2);
        let goal_neq = teq(list1, list3);
        assert!(prover.check_goal(&goal_eq));
        assert!(!prover.check_goal(&goal_neq)); // This check needs care - absence of proof != proof of inequality
    }

    #[test]
    fn test_product_type_projections() {
        let mut prover = Prover::new();

        // Rule: Proj1(Pair(x, y)) -> x
        let rule1 = RewriteRule {
            name: "proj1".into(),
            lhs: Pattern::Proj1(Box::new(Pattern::Pair(Box::new(pvar("x")), Box::new(pvar("y"))))),
            rhs: pvar("x"),
            condition: None,
        };
        // Rule: Proj2(Pair(x, y)) -> y
        let rule2 = RewriteRule {
            name: "proj2".into(),
            lhs: Pattern::Proj2(Box::new(Pattern::Pair(Box::new(pvar("x")), Box::new(pvar("y"))))),
            rhs: pvar("y"),
            condition: None,
        };
        prover.add_rule(rule1);
        prover.add_rule(rule2);

        // Term: Proj1(Pair(a, b))
        let a = tvar("a");
        let b = tvar("b");
        let pair = term(Term::Pair(Rc::clone(&a), Rc::clone(&b)));
        let proj1_expr = term(Term::Proj1(Rc::clone(&pair)));

        // Add term and saturate
        let _ = prover.egraph.add_term(&proj1_expr);
        prover.saturate();

        // Goal: Proj1(Pair(a, b)) = a
        let goal = teq(proj1_expr, a);
        assert!(prover.check_goal(&goal));
    }

    // --- Quantifier Tests (Conceptual - Require Deeper Implementation) ---
    // A full quantifier implementation is complex. These show the intent.

    // #[test]
    // fn test_forall_instantiation() {
    //     let mut prover = Prover::new();
    //     // Assume type system allows adding terms with types.
    //     // Assume a rule like: ForAll(x, T, P(x)), Term(a: T) |- P(a)
    //
    //     // Axiom: ForAll x:Int. IsEven(x) \/ IsOdd(x)
    //     // Axiom: Term(5: Int)
    //
    //     // Saturate
    //
    //     // Goal: IsEven(5) \/ IsOdd(5)
    //     // assert!(prover.check_goal(&goal));
    // }

    // #[test]
    // fn test_exists_witness() {
    //      let mut prover = Prover::new();
    //      // Assume type system and rules.
    //      // Rule: P(w), Term(w: T) |- Exists x:T. P(x)
    //
    //      // Axiom: IsEven(4)
    //      // Axiom: Term(4: Int)
    //
    //      // Saturate
    //
    //      // Goal: Exists x:Int. IsEven(x)
    //      // assert!(prover.check_goal(&goal));
    // }
}

// You would run tests using `cargo test`
