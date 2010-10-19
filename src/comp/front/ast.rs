
import util.common.option;
import std.map.hashmap;
import std.util.option;
import util.common.span;
import util.common.spanned;

type ident = str;

type name_ = rec(ident ident, vec[@ty] types);
type name = spanned[name_];
type path = vec[name];

type crate_num = int;
type def_num = int;
type def_id = tup(crate_num, def_num);

tag def {
    def_fn(def_id);
    def_mod(def_id);
    def_const(def_id);
    def_arg(def_id);
    def_local(def_id);
    def_ty(def_id);
    def_ty_arg(def_id);
}

type crate = spanned[crate_];
type crate_ = rec(_mod module);

type block = spanned[block_];
type block_ = rec(vec[@stmt] stmts,
                  hashmap[ident,uint] index);

tag binop {
    add;
    sub;
    mul;
    div;
    rem;
    and;
    or;
    bitxor;
    bitand;
    bitor;
    lsl;
    lsr;
    asr;
    eq;
    lt;
    le;
    ne;
    ge;
    gt;
}

tag unop {
    box;
    deref;
    bitnot;
    not;
    neg;
}

type stmt = spanned[stmt_];
tag stmt_ {
    stmt_decl(@decl);
    stmt_ret(option[@expr]);
    stmt_log(@expr);
    stmt_expr(@expr);
}

type local = rec(option[@ty] ty,
                 bool infer,
                 ident ident,
                 option[@expr] init,
                 def_id id);

type decl = spanned[decl_];
tag decl_ {
    decl_local(local);
    decl_item(@item);
}

type expr = spanned[expr_];
tag expr_ {
    expr_vec(vec[@expr], option[@ty]);
    expr_tup(vec[tup(bool /* mutability */, @expr)], option[@ty]);
    expr_rec(vec[tup(ident,@expr)], option[@ty]);
    expr_call(@expr, vec[@expr], option[@ty]);
    expr_binary(binop, @expr, @expr, option[@ty]);
    expr_unary(unop, @expr, option[@ty]);
    expr_lit(@lit, option[@ty]);
    expr_cast(@expr, @ty);
    expr_if(@expr, block, option[block], option[@ty]);
    expr_block(block, option[@ty]);
    expr_assign(@expr /* TODO: @expr|is_lval */, @expr, option[@ty]);
    expr_field(@expr, ident, option[@ty]);
    expr_index(@expr, @expr, option[@ty]);
    expr_name(name, option[def], option[@ty]);
}

type lit = spanned[lit_];
tag lit_ {
    lit_str(str);
    lit_char(char);
    lit_int(int);
    lit_uint(uint);
    lit_nil;
    lit_bool(bool);
}

type ty = spanned[ty_];
tag ty_ {
    ty_nil;
    ty_bool;
    ty_int;
    ty_uint;
    ty_machine(util.common.ty_mach);
    ty_char;
    ty_str;
    ty_box(@ty);
    ty_vec(@ty);
    ty_tup(vec[tup(bool /* mutability */, @ty)]);
    ty_path(path, option[def]);
}

tag mode {
    val;
    alias;
}

type arg = rec(mode mode, @ty ty, ident ident, def_id id);
type _fn = rec(vec[arg] inputs,
               @ty output,
               block body);

type _mod = rec(vec[@item] items,
                hashmap[ident,uint] index);

type item = spanned[item_];
tag item_ {
    item_fn(ident, _fn, def_id);
    item_mod(ident, _mod, def_id);
    item_ty(ident, @ty, def_id);
}


//
// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// compile-command: "make -k -C ../.. 2>&1 | sed -e 's/\\/x\\//x:\\//g'";
// End:
//
