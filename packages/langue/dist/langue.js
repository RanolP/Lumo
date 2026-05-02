const LUMO_TAG = Symbol.for("Lumo/tag");
const __thunk = (fn) => { fn.__t = 1; return fn; };
const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };
const __identity = (__v) => __v;

import { readFileSync as __lumo_readFileSync, writeFileSync as __lumo_writeFileSync } from "node:fs";



export function to_screaming_snake(__caps, name, __k) {
  return __thunk(() => {
    return __k(to_screaming_snake_loop__lto_73ce111b(name, 0, ""));
  });
}

export function to_upper_string(__caps, s, __k) {
  return __thunk(() => {
    return __k(to_upper_string_loop__lto_1fab3ad0(s, 0, ""));
  });
}

export function collect_tokens(__caps, grammar, __k) {
  return __thunk(() => {
    const token_defs = grammar.args[0];
    const attrs = grammar.args[1];
    return collect_tokens_from_rules(__caps, grammar.args[2], List["nil"], List["nil"], (pair) => {
      return dedupe_strings(__caps, pair.args[0], (__cps_v_3) => {
        return sort_strings(__caps, __cps_v_3, (__cps_v_0) => {
          return dedupe_strings(__caps, pair.args[1], (__cps_v_2) => {
            return sort_strings(__caps, __cps_v_2, (__cps_v_1) => {
              return __k(CollectedTokens["mk"](__cps_v_0, __cps_v_1));
            });
          });
        });
      });
    });
  });
}


export const CollectedTokens = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const StringPair = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };

export function collect_tokens_from_rules(__caps, rules, kws, syms, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else {
      const __match_3 = rules.args[0];
      const name = __match_3.args[0];
      return collect_tokens_from_body(__caps, __match_3.args[1], kws, syms, (pair) => {
        return collect_tokens_from_rules(__caps, rules.args[1], pair.args[0], pair.args[1], __k);
      });
    }
  });
}

export function collect_tokens_from_body(__caps, body, kws, syms, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return __k(collect_tokens_from_elements(body.args[0], kws, syms));
    } else if ((body[LUMO_TAG] === "alternatives")) {
      return collect_tokens_from_alts__lto_9309ae26(__caps, body.args[0], kws, syms, __k);
    } else {
      const atom_names = body.args[0];
      return __k(collect_tokens_from_pratt_alts(body.args[1], kws, syms));
    }
  });
}

export function collect_tokens_from_pratt_alts(alts, kws, syms) {
  if ((alts[LUMO_TAG] === "nil")) {
    return StringPair["mk"](kws, syms);
  } else {
    const __match_7 = alts.args[0];
    const name = __match_7.args[0];
    const bp = __match_7.args[2];
    const __match_8 = collect_tokens_from_elements(__match_7.args[1], kws, syms);
    return collect_tokens_from_pratt_alts(alts.args[1], __match_8.args[0], __match_8.args[1]);
  }
}

export function collect_alt_token(__caps, name, rest, kws, syms, __k) {
  return __thunk(() => {
    if (has_alpha__lto_090deca7(name, 0)) {
      return collect_tokens_from_alts__lto_9309ae26(__caps, rest, List["cons"](name, kws), syms, __k);
    } else {
      return collect_tokens_from_alts__lto_9309ae26(__caps, rest, kws, List["cons"](name, syms), __k);
    }
  });
}

export function collect_tokens_from_elements(elems, kws, syms) {
  if ((elems[LUMO_TAG] === "nil")) {
    return StringPair["mk"](kws, syms);
  } else {
    const __match_11 = collect_tokens_from_element(elems.args[0], kws, syms);
    return collect_tokens_from_elements(elems.args[1], __match_11.args[0], __match_11.args[1]);
  }
}

export function collect_tokens_from_element(elem, kws, syms) {
  if ((elem[LUMO_TAG] === "token")) {
    const __match_13 = elem.args[0];
    if ((__match_13[LUMO_TAG] === "keyword")) {
      return StringPair["mk"](List["cons"](__match_13.args[0], kws), syms);
    } else if ((__match_13[LUMO_TAG] === "symbol")) {
      return StringPair["mk"](kws, List["cons"](__match_13.args[0], syms));
    } else {
      const n = __match_13.args[0];
      return StringPair["mk"](kws, syms);
    }
  } else if ((elem[LUMO_TAG] === "node")) {
    const ref = elem.args[0];
    return StringPair["mk"](kws, syms);
  } else {
    return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
      return collect_tokens_from_element(elem.args[1], kws, syms);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
      return collect_tokens_from_element(inner, kws, syms);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
      return collect_tokens_from_element(inner, kws, syms);
    })(elem.args[0]) : ((elems) => {
      return collect_tokens_from_elements(elems, kws, syms);
    })(elem.args[0]))));
  }
}

export function dedupe_strings(__caps, xs, __k) {
  return dedupe_strings_acc(__caps, xs, List["nil"], __k);
}

export function dedupe_strings_acc(__caps, xs, acc, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(acc);
    } else {
      const x = xs.args[0];
      const rest = xs.args[1];
      if (list_contains_string__lto_3890158f(acc, x)) {
        return dedupe_strings_acc(__caps, rest, acc, __k);
      } else {
        return dedupe_strings_acc(__caps, rest, List["cons"](x, acc), __k);
      }
    }
  });
}

export function sort_strings(__caps, xs, __k) {
  return sort_strings_acc(__caps, xs, List["nil"], __k);
}

export function sort_strings_acc(__caps, xs, sorted, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(sorted);
    } else {
      return insert_sorted(__caps, xs.args[0], sorted, (__cps_v_4) => {
        return sort_strings_acc(__caps, xs.args[1], __cps_v_4, __k);
      });
    }
  });
}

export function insert_sorted(__caps, s, xs, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(List["cons"](s, xs));
    } else {
      const x = xs.args[0];
      return string_lt(__caps, s, x, (__cps_v_6) => {
        if (__cps_v_6) {
          return __k(List["cons"](s, xs));
        } else {
          return insert_sorted(__caps, s, xs.args[1], (__cps_v_5) => {
            return __k(List["cons"](x, __cps_v_5));
          });
        }
      });
    }
  });
}

export function string_lt(__caps, s1, s2, __k) {
  return __thunk(() => {
    return __k(string_lt_loop__lto_090deca7(s1, s2, 0));
  });
}

export function emit_pratt_alt_kinds(__caps, s, alts, __k) {
  return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, s, alts, List["nil"], __k);
}

export function emit_ast_rules(__caps, s, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __match_20 = rules.args[0];
      const name = __match_20.args[0];
      const __k_14 = (s2) => {
        return emit_ast_rules(__caps, s2, token_defs, rules.args[1], __k);
      };
      const __match_21 = __match_20.args[1];
      if ((__match_21[LUMO_TAG] === "sequence")) {
        return emit_struct_node__lto_1ba4622a(__caps, s, name, __match_21.args[0], token_defs, __k_14);
      } else if ((__match_21[LUMO_TAG] === "alternatives")) {
        const alts = __match_21.args[0];
        if (is_token_only_alternatives__lto_9309ae26(alts)) {
          return emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k_14);
        } else {
          return __k_14(emit_enum_node__lto_1ba4622a(s, name, alts));
        }
      } else {
        return emit_pratt_rule(__caps, s, name, __match_21.args[0], __match_21.args[1], token_defs, __k_14);
      }
    }
  });
}

export function emit_pratt_rule(__caps, s, rule_name, atom_names, alts, token_defs, __k) {
  return emit_pratt_alt_structs(__caps, s, alts, token_defs, (s2) => {
    return collect_pratt_alt_names(__caps, alts, (__cps_v_7) => {
      return __k(emit_enum_from_names__lto_1ba4622a(s2, rule_name, list_concat_string(atom_names, __cps_v_7)));
    });
  });
}

export function emit_pratt_alt_structs(__caps, s, alts, token_defs, __k) {
  return emit_pratt_alt_structs_dedup(__caps, s, alts, token_defs, List["nil"], __k);
}

export function emit_pratt_alt_structs_dedup(__caps, s, alts, token_defs, seen, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_24 = alts.args[0];
      const name = __match_24.args[0];
      const bp = __match_24.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return emit_pratt_alt_structs_dedup(__caps, s, rest, token_defs, seen, __k);
      } else {
        return emit_struct_node__lto_1ba4622a(__caps, s, name, __match_24.args[1], token_defs, (s2) => {
          return emit_pratt_alt_structs_dedup(__caps, s2, rest, token_defs, List["cons"](name, seen), __k);
        });
      }
    }
  });
}

export function collect_pratt_alt_names(__caps, alts, __k) {
  return collect_pratt_alt_names_dedup(__caps, alts, List["nil"], __k);
}

export function collect_pratt_alt_names_dedup(__caps, alts, seen, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const rest = alts.args[1];
      const __match_27 = alts.args[0];
      const name = __match_27.args[0];
      const elems = __match_27.args[1];
      const bp = __match_27.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return collect_pratt_alt_names_dedup(__caps, rest, seen, __k);
      } else {
        return collect_pratt_alt_names_dedup(__caps, rest, List["cons"](name, seen), (__cps_v_8) => {
          return __k(List["cons"](name, __cps_v_8));
        });
      }
    }
  });
}

export function has_labeled_elements(elems) {
  if ((elems[LUMO_TAG] === "nil")) {
    return false;
  } else if ((elems.args[0][LUMO_TAG] === "labeled")) {
    return true;
  } else {
    return has_labeled_elements(elems.args[1]);
  }
}

export function emit_accessors_for_elements(__caps, s, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = elems.args[1];
      const __match_32 = elems.args[0];
      if ((__match_32[LUMO_TAG] === "labeled")) {
        return emit_single_accessor(__caps, s, __match_32.args[0], __match_32.args[1], token_defs, (s2) => {
          return emit_accessors_for_elements(__caps, s2, rest, token_defs, __k);
        });
      } else {
        return emit_accessors_for_elements(__caps, s, rest, token_defs, __k);
      }
    }
  });
}

export function emit_single_accessor(__caps, s, label, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      return emit_token_accessor__lto_1ba4622a(__caps, s, label, elem.args[0], false, __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return emit_token_accessor__lto_1ba4622a(__caps, s, label, TokenRef["named"](name), false, __k);
      } else {
        return __k(emit_node_accessor__lto_1ba4622a(s, label, name, false));
      }
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_single_accessor(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "labeled") ? ((inner) => {
        return emit_single_accessor(__caps, s, label, inner, token_defs, __k);
      })(elem.args[1]) : ((elems) => {
        return __k(s);
      })(elem.args[0]))));
    }
  });
}

export function emit_single_accessor_repeated(__caps, s, label, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      return emit_token_accessor__lto_1ba4622a(__caps, s, label, elem.args[0], true, __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return emit_token_accessor__lto_1ba4622a(__caps, s, label, TokenRef["named"](name), true, __k);
      } else {
        return __k(emit_node_accessor__lto_1ba4622a(s, label, name, true));
      }
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "labeled") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[1]) : ((elems) => {
        return __k(s);
      })(elem.args[0]))));
    }
  });
}

export function token_kind_from_ref(__caps, t, __k) {
  return __thunk(() => {
    if ((t[LUMO_TAG] === "named")) {
      return to_screaming_snake(__caps, t.args[0], __k);
    } else if ((t[LUMO_TAG] === "keyword")) {
      return keyword_variant__lto_1ba4622a(__caps, t.args[0], __k);
    } else {
      return __k(symbol_variant__lto_8227044e(t.args[0]));
    }
  });
}

export function emit_parse_rules(__caps, s, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __match_41 = rules.args[0];
      const name = __match_41.args[0];
      const body = __match_41.args[1];
      return emit_can_parse_method__lto_1ba4622a(__caps, s, name, body, token_defs, (s2) => {
        return emit_parse_rule(__caps, s2, name, body, token_defs, (s3) => {
          return emit_parse_rules(__caps, s3, token_defs, rules.args[1], __k);
        });
      });
    }
  });
}

export function make_body_lookahead(__caps, body, token_defs, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return make_first_elem_lookahead(__caps, body.args[0], token_defs, __k);
    } else if ((body[LUMO_TAG] === "alternatives")) {
      return make_alts_lookahead__lto_1ba4622a(__caps, body.args[0], __k);
    } else {
      return make_pratt_lookahead__lto_8227044e(__caps, body.args[0], body.args[1], __k);
    }
  });
}

export function make_first_elem_lookahead(__caps, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const elem = elems.args[0];
      const rest = elems.args[1];
      const __match_44 = unwrap_labeled_elem(elem);
      if ((__match_44[LUMO_TAG] === "optional")) {
        const inner = __match_44.args[0];
        return make_first_elem_lookahead(__caps, rest, token_defs, __k);
      } else if ((__match_44[LUMO_TAG] === "repeated")) {
        const inner = __match_44.args[0];
        return make_first_elem_lookahead(__caps, rest, token_defs, __k);
      } else {
        return make_element_lookahead__lto_1ba4622a(__caps, elem, token_defs, __k);
      }
    }
  });
}

export function unwrap_labeled_elem(elem) {
  if ((elem[LUMO_TAG] === "labeled")) {
    const label = elem.args[0];
    return elem.args[1];
  } else {
    return elem;
  }
}

export function emit_parse_rule(__caps, s, name, body, token_defs, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return emit_parse_sequence_rule__lto_8227044e(__caps, s, name, body.args[0], token_defs, __k);
    } else if ((body[LUMO_TAG] === "alternatives")) {
      const alts = body.args[0];
      if (is_token_only_alternatives__lto_9309ae26(alts)) {
        return __k(s);
      } else {
        return emit_parse_alt_rule__lto_1ba4622a(__caps, s, name, alts, __k);
      }
    } else {
      return emit_parse_pratt_rule__lto_1ba4622a(__caps, s, name, body.args[0], body.args[1], token_defs, __k);
    }
  });
}

export function emit_pratt_at_predicates(__caps, s, alts, token_defs, __k) {
  return emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, s, alts, token_defs, List["nil"], __k);
}

export function emit_parse_elements(__caps, s, elems, token_defs, indent, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return emit_parse_element__lto_1ba4622a(__caps, s, elems.args[0], token_defs, indent, (s2) => {
        return emit_parse_elements(__caps, s2, elems.args[1], token_defs, indent, __k);
      });
    }
  });
}

export function make_group_lookahead(__caps, elems, token_defs, __k) {
  return make_first_elem_lookahead(__caps, elems, token_defs, __k);
}

export function to_snake(__caps, name, __k) {
  return __thunk(() => {
    return __k(to_snake_loop__lto_1fab3ad0(name, 0, ""));
  });
}

export function list_concat_elem(xs, ys) {
  if ((xs[LUMO_TAG] === "nil")) {
    return ys;
  } else {
    return List["cons"](xs.args[0], list_concat_elem(xs.args[1], ys));
  }
}


export const GrammarAttr = { "parser_generate": (arg0) => {
  return { [LUMO_TAG]: "parser_generate", args: [arg0] };
} };


export const Grammar = { "mk": (arg0, arg1, arg2) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1, arg2] };
} };


export const Rule = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const RuleBody = { "sequence": (arg0) => {
  return { [LUMO_TAG]: "sequence", args: [arg0] };
}, "alternatives": (arg0) => {
  return { [LUMO_TAG]: "alternatives", args: [arg0] };
}, "pratt": (arg0, arg1) => {
  return { [LUMO_TAG]: "pratt", args: [arg0, arg1] };
} };


export const PrattAlt = { "mk": (arg0, arg1, arg2) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1, arg2] };
} };


export const BindingPower = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const BpVal = { "none": { [LUMO_TAG]: "none" }, "num": (arg0) => {
  return { [LUMO_TAG]: "num", args: [arg0] };
} };


export const Alternative = { "mk": (arg0) => {
  return { [LUMO_TAG]: "mk", args: [arg0] };
} };


export const Element = { "token": (arg0) => {
  return { [LUMO_TAG]: "token", args: [arg0] };
}, "node": (arg0) => {
  return { [LUMO_TAG]: "node", args: [arg0] };
}, "labeled": (arg0, arg1) => {
  return { [LUMO_TAG]: "labeled", args: [arg0, arg1] };
}, "optional": (arg0) => {
  return { [LUMO_TAG]: "optional", args: [arg0] };
}, "repeated": (arg0) => {
  return { [LUMO_TAG]: "repeated", args: [arg0] };
}, "group": (arg0) => {
  return { [LUMO_TAG]: "group", args: [arg0] };
} };


export const TokenRef = { "keyword": (arg0) => {
  return { [LUMO_TAG]: "keyword", args: [arg0] };
}, "symbol": (arg0) => {
  return { [LUMO_TAG]: "symbol", args: [arg0] };
}, "named": (arg0) => {
  return { [LUMO_TAG]: "named", args: [arg0] };
} };


export const NodeRef = { "mk": (arg0) => {
  return { [LUMO_TAG]: "mk", args: [arg0] };
} };

function __main_cps(__caps, __k) {
  return run__lto_3829b133(__caps, __k);
}

export function main() {
  return __trampoline(__main_cps({ IO_IO: IO(__identity), FS_FS: FS(__identity), Process_Process: Process(__identity), StrOps_StrOps: StrOps(__identity), Add_String: __impl_String_Add(__identity), Mul_Number: __impl_Number_Mul(__identity), NumOps_NumOps: NumOps(__identity), Sub_Number: __impl_Number_Sub(__identity), Add_Number: __impl_Number_Add(__identity), PartialEq_String: __impl_String_PartialEq(__identity), PartialOrd_Number: __impl_Number_PartialOrd(__identity) }, __identity));
}

export function find_parser_path(attrs) {
  if ((attrs[LUMO_TAG] === "nil")) {
    return "";
  } else {
    const rest = attrs.args[1];
    return attrs.args[0].args[0];
  }
}


export const ParseState = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const ParseResult = { "ok": (arg0, arg1) => {
  return { [LUMO_TAG]: "ok", args: [arg0, arg1] };
}, "err": (arg0, arg1) => {
  return { [LUMO_TAG]: "err", args: [arg0, arg1] };
} };

export function is_ident_start(__caps, c, __k) {
  return __thunk(() => {
    return __k(is_alpha__lto_9309ae26(c));
  });
}

export function state_src(st) {
  const pos = st.args[1];
  return st.args[0];
}

export function state_pos(st) {
  const src = st.args[0];
  return st.args[1];
}

export function scan_ident_rest(__caps, st, __k) {
  return __thunk(() => {
    if (state_eof__lto_9309ae26(st)) {
      return __k(st);
    } else if (is_ident_continue__lto_3890158f(state_peek__lto_9309ae26(st))) {
      return scan_ident_rest(__caps, state_advance__lto_92991de6(st, 1), __k);
    } else {
      return __k(st);
    }
  });
}

export function peek_char(__caps, st, __k) {
  return __thunk(() => {
    return __k(state_peek__lto_9309ae26(skip_ws__lto_1bb67705(st)));
  });
}

export function peek_is_pratt(__caps, st, __k) {
  return __thunk(() => {
    return __k(peek_is_word__lto_1bb67705(st, "pratt"));
  });
}

export function try_parse_bp(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (peek_is_bp_marker__lto_3890158f(st2)) {
      const __match_57 = expect__lto_f3280589(state_advance__lto_92991de6(st2, 2), "(");
      if ((__match_57[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_57.args[0], __match_57.args[1]));
      } else {
        return parse_bp_val(__caps, __match_57.args[1], (__cps_v_9) => {
          if ((__cps_v_9[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_9.args[0], __cps_v_9.args[1]));
          } else {
            const __match_59 = expect__lto_f3280589(__cps_v_9.args[1], ")");
            if ((__match_59[LUMO_TAG] === "err")) {
              return __k(ParseResult["err"](__match_59.args[0], __match_59.args[1]));
            } else {
              return __k(ParseResult["ok"](__cps_v_9.args[0], __match_59.args[1]));
            }
          }
        });
      }
    } else {
      return __k(ParseResult["err"]("no bp marker", state_pos(st2)));
    }
  });
}

export function classify_literal(__caps, text, __k) {
  return __thunk(() => {
    if (has_alpha__lto_090deca7(text, 0)) {
      return __k(TokenRef["keyword"](text));
    } else {
      return __k(TokenRef["symbol"](text));
    }
  });
}

export function parse_grammar(__caps, src, __k) {
  return __thunk(() => {
    return parse_grammar_items__lto_3890158f(__caps, ParseState["mk"](src, 0), List["nil"], List["nil"], List["nil"], __k);
  });
}

export function parse_token_def(__caps, st, __k) {
  return __thunk(() => {
    const __match_61 = expect__lto_f3280589(st, "@token");
    if ((__match_61[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_61.args[0], __match_61.args[1]));
    } else {
      return parse_token_names__lto_3890158f(__caps, __match_61.args[1], List["nil"], __k);
    }
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident__lto_1ba4622a(__caps, st, (__cps_v_11) => {
    if ((__cps_v_11[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_11.args[0], __cps_v_11.args[1]));
    } else {
      const name = __cps_v_11.args[0];
      const __match_63 = expect__lto_f3280589(__cps_v_11.args[1], "=");
      if ((__match_63[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_63.args[0], __match_63.args[1]));
      } else {
        return parse_rule_body__lto_3890158f(__caps, __match_63.args[1], name, (__cps_v_10) => {
          if ((__cps_v_10[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_10.args[0], __cps_v_10.args[1]));
          } else {
            return __k(ParseResult["ok"](Rule["mk"](name, __cps_v_10.args[0]), __cps_v_10.args[1]));
          }
        });
      }
    }
  });
}

export function parse_pratt_body(__caps, st, __k) {
  return __thunk(() => {
    const __match_65 = expect__lto_f3280589(st, "pratt");
    if ((__match_65[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_65.args[0], __match_65.args[1]));
    } else {
      const __match_66 = expect__lto_f3280589(__match_65.args[1], "{");
      if ((__match_66[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_66.args[0], __match_66.args[1]));
      } else {
        return parse_pratt_items__lto_3890158f(__caps, __match_66.args[1], List["nil"], List["nil"], (__cps_v_12) => {
          if ((__cps_v_12[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_12.args[0], __cps_v_12.args[1]));
          } else {
            const __match_68 = expect__lto_f3280589(__cps_v_12.args[1], "}");
            if ((__match_68[LUMO_TAG] === "err")) {
              return __k(ParseResult["err"](__match_68.args[0], __match_68.args[1]));
            } else {
              return __k(ParseResult["ok"](__cps_v_12.args[0], __match_68.args[1]));
            }
          }
        });
      }
    }
  });
}

export function parse_pratt_alt_body(__caps, st, name, __k) {
  return try_parse_bp(__caps, st, (lbp_res) => {
    if ((lbp_res[LUMO_TAG] === "ok")) {
      const lbp = lbp_res.args[0];
      return parse_pratt_pattern__lto_3890158f(__caps, lbp_res.args[1], List["nil"], (__cps_v_16) => {
        if ((__cps_v_16[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_16.args[0], __cps_v_16.args[1]));
        } else {
          const elems = __cps_v_16.args[0];
          const st3 = __cps_v_16.args[1];
          return try_parse_bp(__caps, st3, (__cps_v_15) => {
            if ((__cps_v_15[LUMO_TAG] === "ok")) {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](lbp, __cps_v_15.args[0])), __cps_v_15.args[1]));
            } else {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](lbp, BpVal["none"])), st3));
            }
          });
        }
      });
    } else {
      return parse_pratt_pattern__lto_3890158f(__caps, st, List["nil"], (__cps_v_14) => {
        if ((__cps_v_14[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_14.args[0], __cps_v_14.args[1]));
        } else {
          const elems = __cps_v_14.args[0];
          const st2 = __cps_v_14.args[1];
          return try_parse_bp(__caps, st2, (__cps_v_13) => {
            if ((__cps_v_13[LUMO_TAG] === "ok")) {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](BpVal["none"], __cps_v_13.args[0])), __cps_v_13.args[1]));
            } else {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](BpVal["none"], BpVal["none"])), st2));
            }
          });
        }
      });
    }
  });
}

export function parse_bp_val(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (peek_is_word__lto_1bb67705(st2, "None")) {
      return __k(ParseResult["ok"](BpVal["none"], state_advance__lto_92991de6(st2, 4)));
    } else {
      return parse_number__lto_1ba4622a(__caps, st2, (__cps_v_17) => {
        if ((__cps_v_17[LUMO_TAG] === "ok")) {
          return __k(ParseResult["ok"](BpVal["num"](__cps_v_17.args[0]), __cps_v_17.args[1]));
        } else {
          return __k(ParseResult["err"](__cps_v_17.args[0], __cps_v_17.args[1]));
        }
      });
    }
  });
}

export function scan_digits(__caps, st, __k) {
  return __thunk(() => {
    if (state_eof__lto_9309ae26(st)) {
      return __k(st);
    } else if (is_digit__lto_9309ae26(state_peek__lto_9309ae26(st))) {
      return scan_digits(__caps, state_advance__lto_92991de6(st, 1), __k);
    } else {
      return __k(st);
    }
  });
}

export function parse_alternatives(__caps, st, __k) {
  return parse_alt_items__lto_3890158f(__caps, st, List["nil"], __k);
}

export function parse_sequence(__caps, st, __k) {
  return parse_seq_elements(__caps, st, List["nil"], __k);
}

export function parse_seq_elements(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
    } else {
      return is_seq_terminator__lto_3890158f(__caps, st2, (__cps_v_19) => {
        if (__cps_v_19) {
          return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
        } else {
          return parse_element(__caps, st2, (__cps_v_18) => {
            if ((__cps_v_18[LUMO_TAG] === "ok")) {
              return parse_seq_elements(__caps, __cps_v_18.args[1], List["cons"](__cps_v_18.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_18.args[0], __cps_v_18.args[1]));
            }
          });
        }
      });
    }
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom__lto_3890158f(__caps, st, (__cps_v_20) => {
    if ((__cps_v_20[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_20.args[0], __cps_v_20.args[1]));
    } else {
      return __k(apply_postfix_elem__lto_3890158f(__cps_v_20.args[0], __cps_v_20.args[1]));
    }
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    const token_defs = g.args[0];
    return resolve_rules(__caps, token_defs, g.args[2], (__cps_v_21) => {
      return __k(Grammar["mk"](token_defs, g.args[1], __cps_v_21));
    });
  });
}

export function resolve_rules(__caps, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const __match_84 = rules.args[0];
      return resolve_body(__caps, token_defs, __match_84.args[1], (resolved_body) => {
        return resolve_rules(__caps, token_defs, rules.args[1], (__cps_v_22) => {
          return __k(List["cons"](Rule["mk"](__match_84.args[0], resolved_body), __cps_v_22));
        });
      });
    }
  });
}

export function resolve_body(__caps, token_defs, body, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return resolve_elements(__caps, token_defs, body.args[0], (__cps_v_24) => {
        return __k(RuleBody["sequence"](__cps_v_24));
      });
    } else if ((body[LUMO_TAG] === "alternatives")) {
      const alts = body.args[0];
      return __k(body);
    } else {
      return resolve_pratt_alts(__caps, token_defs, body.args[1], (__cps_v_23) => {
        return __k(RuleBody["pratt"](body.args[0], __cps_v_23));
      });
    }
  });
}

export function resolve_pratt_alts(__caps, token_defs, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const __match_87 = alts.args[0];
      return resolve_elements(__caps, token_defs, __match_87.args[1], (__cps_v_27) => {
        const __cps_v_25 = PrattAlt["mk"](__match_87.args[0], __cps_v_27, __match_87.args[2]);
        return resolve_pratt_alts(__caps, token_defs, alts.args[1], (__cps_v_26) => {
          return __k(List["cons"](__cps_v_25, __cps_v_26));
        });
      });
    }
  });
}

export function resolve_elements(__caps, token_defs, elems, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      return resolve_element(__caps, token_defs, elems.args[0], (__cps_v_28) => {
        return resolve_elements(__caps, token_defs, elems.args[1], (__cps_v_29) => {
          return __k(List["cons"](__cps_v_28, __cps_v_29));
        });
      });
    }
  });
}

export function resolve_element(__caps, token_defs, elem, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const t = elem.args[0];
      return __k(elem);
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return __k(Element["token"](TokenRef["named"](name)));
      } else {
        return __k(elem);
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        return resolve_element(__caps, token_defs, elem.args[1], (__cps_v_33) => {
          return __k(Element["labeled"](label, __cps_v_33));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_32) => {
          return __k(Element["optional"](__cps_v_32));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_31) => {
          return __k(Element["repeated"](__cps_v_31));
        });
      })(elem.args[0]) : ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_30) => {
          return __k(Element["group"](__cps_v_30));
        });
      })(elem.args[0]))));
    }
  });
}

export function list_reverse_string(xs) {
  return list_reverse_string_acc(xs, List["nil"]);
}

export function list_reverse_string_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_string_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}

export function list_reverse_rule(xs) {
  return list_reverse_rule_acc(xs, List["nil"]);
}

export function list_reverse_rule_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_rule_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}

export function list_reverse_alt(xs) {
  return list_reverse_alt_acc(xs, List["nil"]);
}

export function list_reverse_alt_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_alt_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}

export function list_reverse_elem(xs) {
  return list_reverse_elem_acc(xs, List["nil"]);
}

export function list_reverse_elem_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_elem_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}

export function list_reverse_pratt_alt(xs) {
  return list_reverse_pratt_alt_acc(xs, List["nil"]);
}

export function list_reverse_pratt_alt_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_pratt_alt_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}

export function list_concat_string(xs, ys) {
  if ((xs[LUMO_TAG] === "nil")) {
    return ys;
  } else {
    return List["cons"](xs.args[0], list_concat_string(xs.args[1], ys));
  }
}

export function list_reverse_attr(xs) {
  return list_reverse_attr_acc(xs, List["nil"]);
}

export function list_reverse_attr_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else {
    return list_reverse_attr_acc(xs.args[1], List["cons"](xs.args[0], acc));
  }
}


export const Bool = { "true": true, "false": false };


export const __impl_String_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.concat(__caps, self, other, (__cps_v_34) => {
        return __k_handle(__k_perform(__cps_v_34));
      });
    });
  } };
};

export const __impl_String_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.eq(__caps, self, other, (__cps_v_35) => {
        return __k_handle(__k_perform(__cps_v_35));
      });
    });
  } };
};

export const String = { len: (self) => {
  return self.length;
}, char_at: (self, idx) => {
  return self.charAt(idx);
}, slice: (self, start, end) => {
  return __str_slice(self, start, end);
}, starts_with: (self, prefix) => {
  return __str_starts_with(self, prefix);
}, contains: (self, sub) => {
  return __str_contains(self, sub);
}, index_of: (self, sub) => {
  return __str_index_of(self, sub);
}, trim: (self) => {
  return __str_trim(self);
}, char_code_at: (self, idx) => {
  return __char_code_at(self, idx);
}, replace_all: (self, from, to) => {
  return __str_replace_all(self, from, to);
} };

export const Number = { to_string: (self) => {
  return __num_to_string(self);
}, to_char: (self) => {
  return fromCharCode(self);
} };

export function __str_slice(s, start, end) {
  return s.slice(start, end);
}

export function __str_starts_with(s, prefix) {
  return s.startsWith(prefix);
}

export function __str_contains(s, sub) {
  return s.includes(sub);
}

export function __str_index_of(s, sub) {
  return s.indexOf(sub);
}

export function __str_trim(s) {
  return s.trim();
}

export function __char_code_at(s, idx) {
  return s.charCodeAt(idx);
}

export function __str_replace_all(s, from, to) {
  return s.replaceAll(from, to);
}

export function fromCharCode(code) {
  return globalThis.String.fromCharCode(code);
}

export function __num_to_string(n) {
  return n.toString();
}

export const StrOps = (__k_handle) => {
  return { len: (__caps, s, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((s) => {
        return s.length;
      })(s)));
    });
  }, char_at: (__caps, s, idx, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((s, idx) => {
        return s.charAt(idx);
      })(s, idx)));
    });
  }, slice: (__caps, s, start, end, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_slice(s, start, end)));
    });
  }, concat: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a + b);
      })(a, b)));
    });
  }, eq: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a === b);
      })(a, b)));
    });
  }, starts_with: (__caps, s, prefix, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_starts_with(s, prefix)));
    });
  }, contains: (__caps, s, sub, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_contains(s, sub)));
    });
  }, index_of: (__caps, s, sub, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_index_of(s, sub)));
    });
  }, trim: (__caps, s, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_trim(s)));
    });
  }, char_code_at: (__caps, s, idx, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__char_code_at(s, idx)));
    });
  }, from_char_code: (__caps, code, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(fromCharCode(code)));
    });
  }, replace_all: (__caps, s, from, to, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__str_replace_all(s, from, to)));
    });
  }, num_to_string: (__caps, n, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__num_to_string(n)));
    });
  } };
};


export const Ordering = { "less": { [LUMO_TAG]: "less" }, "equal": { [LUMO_TAG]: "equal" }, "greater": { [LUMO_TAG]: "greater" } };










export const __impl_Bool_Not = (__k_handle) => {
  return { not: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((__match_99) => {
        if (__match_99) {
          return false;
        } else {
          return true;
        }
      })(self)));
    });
  } };
};


export const List = { "nil": { [LUMO_TAG]: "nil" }, "cons": (arg0, arg1) => {
  return { [LUMO_TAG]: "cons", args: [arg0, arg1] };
} };


export const __impl_Number_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.eq(__caps, self, other, (__cps_v_36) => {
        return __k_handle(__k_perform(__cps_v_36));
      });
    });
  } };
};

export const __impl_Number_PartialOrd = (__k_handle) => {
  return { cmp: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.cmp(__caps, self, other, (__cps_v_37) => {
        return __k_handle(__k_perform(__cps_v_37));
      });
    });
  } };
};

export const __impl_Number_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.add(__caps, self, other, (__cps_v_38) => {
        return __k_handle(__k_perform(__cps_v_38));
      });
    });
  } };
};

export const __impl_Number_Sub = (__k_handle) => {
  return { sub: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.sub(__caps, self, other, (__cps_v_39) => {
        return __k_handle(__k_perform(__cps_v_39));
      });
    });
  } };
};

export const __impl_Number_Mul = (__k_handle) => {
  return { mul: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mul(__caps, self, other, (__cps_v_40) => {
        return __k_handle(__k_perform(__cps_v_40));
      });
    });
  } };
};

export const __impl_Number_Div = (__k_handle) => {
  return { div: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.div(__caps, self, other, (__cps_v_41) => {
        return __k_handle(__k_perform(__cps_v_41));
      });
    });
  } };
};

export const __impl_Number_Mod = (__k_handle) => {
  return { mod_: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mod_(__caps, self, other, (__cps_v_42) => {
        return __k_handle(__k_perform(__cps_v_42));
      });
    });
  } };
};

export const __impl_Number_Neg = (__k_handle) => {
  return { neg: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.neg(__caps, self, (__cps_v_43) => {
        return __k_handle(__k_perform(__cps_v_43));
      });
    });
  } };
};

export const NumOps = (__k_handle) => {
  return { add: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a + b);
      })(a, b)));
    });
  }, sub: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a - b);
      })(a, b)));
    });
  }, mul: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a * b);
      })(a, b)));
    });
  }, div: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a / b);
      })(a, b)));
    });
  }, mod_: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return globalThis["_%_"];
      })(a, b)));
    });
  }, neg: (__caps, a, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a) => {
        return (-a);
      })(a)));
    });
  }, floor: (__caps, a, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a) => {
        return Math.floor(a);
      })(a)));
    });
  }, eq: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((a, b) => {
        return (a === b);
      })(a, b)));
    });
  }, cmp: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((__match_100) => {
        if (__match_100) {
          return Ordering["less"];
        } else if ((a === b)) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a < b);
      })(a, b))));
    });
  } };
};


export const IO = (__k_handle) => {
  return { println: (__caps, msg, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((msg) => {
        return globalThis.console.log(msg);
      })(msg)));
    });
  } };
};


export function readFileSync(path, encoding) {
  return __lumo_readFileSync(path, encoding);
}

export function writeFileSync(path, content, encoding) {
  return __lumo_writeFileSync(path, content, encoding);
}

export const FS = (__k_handle) => {
  return { read_file: (__caps, path, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(readFileSync(path, "utf8")));
    });
  }, write_file: (__caps, path, content, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(writeFileSync(path, content, "utf8")));
    });
  } };
};


export function __argv_at_raw(idx) {
  return globalThis.process.argv.at(idx);
}

export function __argv_length_raw() {
  return globalThis.process.argv.length;
}

export function __exit_process(code) {
  return globalThis.process.exit(code);
}

export function __console_error(msg) {
  return globalThis.console.error(msg);
}

export const Process = (__k_handle) => {
  return { arg_at: (__caps, idx, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__argv_at_offset()(idx)));
    });
  }, args_count: (__caps, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__args_count_offset()));
    });
  }, exit_process: (__caps, code, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(__exit_process(code)));
    });
  }, panic_with: (__caps, msg, __k_perform) => {
    return __thunk(() => {
      const _err = __console_error(msg);
      return __k_handle(__k_perform(__exit_process(1)));
    });
  } };
};

export function to_screaming_snake_loop__lto_73ce111b(name, i, acc) {
  const __lto_b_23 = String.len(name);
  const __match_104 = ((i < __lto_b_23) ? Ordering["less"] : ((__match_103) => {
    if (__match_103) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_23)));
  if (((__match_104[LUMO_TAG] === "less") ? false : ((__match_104[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __match_129 = ((code < 65) ? Ordering["less"] : ((__match_128) => {
      if (__match_128) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_129[LUMO_TAG] === "less") ? false : ((__match_129[LUMO_TAG] === "equal") ? true : true)) ? ((__match_133) => {
      if ((__match_133[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_133[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_28) => {
      const __lto_other_29 = 90;
      const __match_131 = (__lto_self_28 < __lto_other_29);
      if (__match_131) {
        return Ordering["less"];
      } else {
        const __match_132 = (__lto_self_28 === __lto_other_29);
        if (__match_132) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      let __match_110;
      let __match_109;
      if ((0 < i)) {
        __match_109 = Ordering["less"];
      } else if ((0 === i)) {
        __match_109 = Ordering["equal"];
      } else {
        __match_109 = Ordering["greater"];
      }
      if ((__match_109[LUMO_TAG] === "less")) {
        __match_110 = true;
      } else if ((__match_109[LUMO_TAG] === "equal")) {
        __match_110 = false;
      } else {
        __match_110 = false;
      }
      if (__match_110) {
        const prev_code = String.char_code_at(String.char_at(name, ((__lto_self_36) => {
          return (__lto_self_36 - 1);
        })(i)), 0);
        const __match_122 = ((prev_code < 97) ? Ordering["less"] : ((__match_121) => {
          if (__match_121) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 97)));
        const __match_115 = ((prev_code < 48) ? Ordering["less"] : ((__match_114) => {
          if (__match_114) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 48)));
        if ((((__match_122[LUMO_TAG] === "less") ? false : ((__match_122[LUMO_TAG] === "equal") ? true : true)) ? ((__match_126) => {
          if ((__match_126[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_126[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_44) => {
          const __lto_other_45 = 122;
          const __match_124 = (__lto_self_44 < __lto_other_45);
          if (__match_124) {
            return Ordering["less"];
          } else {
            const __match_125 = (__lto_self_44 === __lto_other_45);
            if (__match_125) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_56) => {
            return (__lto_self_56 + 1);
          })(i), ((__lto_self_60) => {
            return (__lto_self_60 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_62) => {
            return (__lto_self_62 + "_");
          })(acc)));
        } else if ((((__match_115[LUMO_TAG] === "less") ? false : ((__match_115[LUMO_TAG] === "equal") ? true : true)) ? ((__match_119) => {
          if ((__match_119[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_119[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_52) => {
          const __lto_other_53 = 57;
          const __match_117 = (__lto_self_52 < __lto_other_53);
          if (__match_117) {
            return Ordering["less"];
          } else {
            const __match_118 = (__lto_self_52 === __lto_other_53);
            if (__match_118) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_68) => {
            return (__lto_self_68 + 1);
          })(i), ((__lto_self_72) => {
            return (__lto_self_72 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_74) => {
            return (__lto_self_74 + "_");
          })(acc)));
        } else {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_80) => {
            return (__lto_self_80 + 1);
          })(i), ((__lto_self_84) => {
            return (__lto_self_84 + to_upper_char__lto_f0f5f7cb(c));
          })(acc));
        }
      } else {
        return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_88) => {
          return (__lto_self_88 + 1);
        })(i), ((__lto_self_92) => {
          return (__lto_self_92 + to_upper_char__lto_f0f5f7cb(c));
        })(acc));
      }
    } else {
      return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_96) => {
        return (__lto_self_96 + 1);
      })(i), ((__lto_self_100) => {
        return (__lto_self_100 + to_upper_char__lto_f0f5f7cb(c));
      })(acc));
    }
  }
}

export function to_upper_char__lto_f0f5f7cb(c) {
  const code = String.char_code_at(c, 0);
  const __match_136 = ((code < 97) ? Ordering["less"] : ((__match_135) => {
    if (__match_135) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_136[LUMO_TAG] === "less") ? false : ((__match_136[LUMO_TAG] === "equal") ? true : true))) {
    let __match_141;
    let __match_140;
    if ((code < 122)) {
      __match_140 = Ordering["less"];
    } else if ((code === 122)) {
      __match_140 = Ordering["equal"];
    } else {
      __match_140 = Ordering["greater"];
    }
    if ((__match_140[LUMO_TAG] === "less")) {
      __match_141 = true;
    } else if ((__match_140[LUMO_TAG] === "equal")) {
      __match_141 = true;
    } else {
      __match_141 = false;
    }
    if (__match_141) {
      return fromCharCode((code - 32));
    } else {
      return c;
    }
  } else {
    return c;
  }
}

export function keyword_variant__lto_1ba4622a(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__lto_self_117) => {
    return __k(((a, b) => {
      return (a + b);
    })(__lto_self_117, "_KW"));
  });
}

export function to_upper_string_loop__lto_1fab3ad0(s, i, acc) {
  const __lto_b_124 = String.len(s);
  const __match_144 = ((i < __lto_b_124) ? Ordering["less"] : ((__match_143) => {
    if (__match_143) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_124)));
  if (((__match_144[LUMO_TAG] === "less") ? false : ((__match_144[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    return to_upper_string_loop__lto_1fab3ad0(s, ((__lto_self_125) => {
      return (__lto_self_125 + 1);
    })(i), ((__lto_self_129) => {
      return (__lto_self_129 + to_upper_char__lto_f0f5f7cb(String.char_at(s, i)));
    })(acc));
  }
}

export function symbol_variant__lto_8227044e(sym) {
  if ((sym === "#")) {
    return "HASH";
  } else if ((sym === "(")) {
    return "L_PAREN";
  } else if ((sym === ")")) {
    return "R_PAREN";
  } else if ((sym === "[")) {
    return "L_BRACKET";
  } else if ((sym === "]")) {
    return "R_BRACKET";
  } else if ((sym === "{")) {
    return "L_BRACE";
  } else if ((sym === "}")) {
    return "R_BRACE";
  } else if ((sym === ";")) {
    return "SEMICOLON";
  } else if ((sym === ":")) {
    return "COLON";
  } else if ((sym === ",")) {
    return "COMMA";
  } else if ((sym === "=")) {
    return "EQUALS";
  } else if ((sym === ":=")) {
    return "COLON_EQ";
  } else if ((sym === "=>")) {
    return "FAT_ARROW";
  } else if ((sym === "->")) {
    return "ARROW";
  } else if ((sym === ".")) {
    return "DOT";
  } else if ((sym === "+")) {
    return "PLUS";
  } else if ((sym === "-")) {
    return "MINUS";
  } else if ((sym === "*")) {
    return "STAR";
  } else if ((sym === "/")) {
    return "SLASH";
  } else if ((sym === "%")) {
    return "PERCENT";
  } else if ((sym === "!")) {
    return "BANG";
  } else if ((sym === "<")) {
    return "LT";
  } else if ((sym === ">")) {
    return "GT";
  } else if ((sym === "<=")) {
    return "LT_EQ";
  } else if ((sym === ">=")) {
    return "GT_EQ";
  } else if ((sym === "==")) {
    return "EQ_EQ";
  } else if ((sym === "!=")) {
    return "BANG_EQ";
  } else if ((sym === "&&")) {
    return "AMP_AMP";
  } else if ((sym === "||")) {
    return "PIPE_PIPE";
  } else if ((sym === "_")) {
    return "UNDERSCORE";
  } else {
    return ("SYM_" + sym);
  }
}

export function collect_tokens_from_alts__lto_9309ae26(__caps, alts, kws, syms, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else {
      const rest = alts.args[1];
      const name = alts.args[0].args[0];
      const code = String.char_code_at(name, 0);
      const __match_185 = ((code < 65) ? Ordering["less"] : ((__match_184) => {
        if (__match_184) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(code, 65)));
      if (((__match_185[LUMO_TAG] === "less") ? false : ((__match_185[LUMO_TAG] === "equal") ? true : true))) {
        const __match_182 = ((code < 90) ? Ordering["less"] : ((__match_181) => {
          if (__match_181) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(code, 90)));
        if (((__match_182[LUMO_TAG] === "less") ? true : ((__match_182[LUMO_TAG] === "equal") ? true : false))) {
          return collect_tokens_from_alts__lto_9309ae26(__caps, rest, kws, syms, __k);
        } else {
          return collect_alt_token(__caps, name, rest, kws, syms, __k);
        }
      } else {
        return collect_alt_token(__caps, name, rest, kws, syms, __k);
      }
    }
  });
}

export function string_lt_loop__lto_090deca7(s1, s2, idx) {
  const __lto_b_268 = String.len(s1);
  const __match_188 = ((idx < __lto_b_268) ? Ordering["less"] : ((__match_187) => {
    if (__match_187) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(idx, __lto_b_268)));
  if (((__match_188[LUMO_TAG] === "less") ? false : ((__match_188[LUMO_TAG] === "equal") ? true : true))) {
    let __match_205;
    let __match_204;
    const __lto_b_272 = String.len(s2);
    if ((idx < __lto_b_272)) {
      __match_204 = Ordering["less"];
    } else if ((idx === __lto_b_272)) {
      __match_204 = Ordering["equal"];
    } else {
      __match_204 = Ordering["greater"];
    }
    if ((__match_204[LUMO_TAG] === "less")) {
      __match_205 = false;
    } else if ((__match_204[LUMO_TAG] === "equal")) {
      __match_205 = true;
    } else {
      __match_205 = true;
    }
    if (__match_205) {
      return false;
    } else {
      return true;
    }
  } else {
    let __match_193;
    let __match_192;
    const __lto_b_276 = String.len(s2);
    if ((idx < __lto_b_276)) {
      __match_192 = Ordering["less"];
    } else if ((idx === __lto_b_276)) {
      __match_192 = Ordering["equal"];
    } else {
      __match_192 = Ordering["greater"];
    }
    if ((__match_192[LUMO_TAG] === "less")) {
      __match_193 = false;
    } else if ((__match_192[LUMO_TAG] === "equal")) {
      __match_193 = true;
    } else {
      __match_193 = true;
    }
    if (__match_193) {
      return false;
    } else {
      const ca = String.char_code_at(s1, idx);
      const cb = String.char_code_at(s2, idx);
      const __match_196 = ((ca < cb) ? Ordering["less"] : ((__match_195) => {
        if (__match_195) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(ca, cb)));
      if (((__match_196[LUMO_TAG] === "less") ? true : ((__match_196[LUMO_TAG] === "equal") ? false : false))) {
        return true;
      } else {
        let __match_201;
        let __match_200;
        if ((cb < ca)) {
          __match_200 = Ordering["less"];
        } else if ((cb === ca)) {
          __match_200 = Ordering["equal"];
        } else {
          __match_200 = Ordering["greater"];
        }
        if ((__match_200[LUMO_TAG] === "less")) {
          __match_201 = true;
        } else if ((__match_200[LUMO_TAG] === "equal")) {
          __match_201 = false;
        } else {
          __match_201 = false;
        }
        if (__match_201) {
          return false;
        } else {
          return string_lt_loop__lto_090deca7(s1, s2, ((__lto_self_285) => {
            return (__lto_self_285 + 1);
          })(idx));
        }
      }
    }
  }
}

export function is_token_only_alternatives__lto_9309ae26(alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return true;
  } else {
    const code = String.char_code_at(alts.args[0].args[0], 0);
    const __match_211 = ((code < 65) ? Ordering["less"] : ((__match_210) => {
      if (__match_210) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_211[LUMO_TAG] === "less") ? false : ((__match_211[LUMO_TAG] === "equal") ? true : true)) ? ((__match_215) => {
      if ((__match_215[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_215[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_293) => {
      const __lto_other_294 = 90;
      const __match_213 = (__lto_self_293 < __lto_other_294);
      if (__match_213) {
        return Ordering["less"];
      } else {
        const __match_214 = (__lto_self_293 === __lto_other_294);
        if (__match_214) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      return false;
    } else {
      return is_token_only_alternatives__lto_9309ae26(alts.args[1]);
    }
  }
}

export function generate_syntax_kind__lto_1ba4622a(__caps, grammar, __k) {
  return collect_tokens(__caps, grammar, (collected) => {
    const keywords = collected.args[0];
    const symbols = collected.args[1];
    const attrs = grammar.args[1];
    return emit_named_tokens__lto_1ba4622a(__caps, (((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n\n") + "#[allow(non_camel_case_types)]\n") + "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n") + "#[repr(u16)]\n") + "pub enum SyntaxKind {\n") + "    // Named tokens\n"), grammar.args[0], (s) => {
      return emit_keywords__lto_1ba4622a(__caps, ((s + "    // Trivia\n") + "    WHITESPACE,\n    NEWLINE,\n    UNKNOWN,\n"), keywords, (s) => {
        return emit_node_kinds__lto_1ba4622a(__caps, (emit_symbols__lto_1ba4622a(s, symbols) + "    // Nodes\n"), grammar.args[2], (s) => {
          return emit_from_keyword__lto_1ba4622a(__caps, ((((((s + "    // Sentinel\n    ERROR,\n") + "}\n") + "\nimpl SyntaxKind {\n") + "    pub fn is_trivia(self) -> bool {\n") + "        matches!(self, Self::WHITESPACE | Self::NEWLINE)\n") + "    }\n"), keywords, (s) => {
            return __k((emit_from_symbol__lto_1ba4622a(s, symbols) + "}\n"));
          });
        });
      });
    });
  });
}

export function emit_named_tokens__lto_1ba4622a(__caps, s, tokens, __k) {
  return __thunk(() => {
    if ((tokens[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return to_screaming_snake(__caps, tokens.args[0], (__lto_other_364) => {
        return emit_named_tokens__lto_1ba4622a(__caps, (((s + "    ") + __lto_other_364) + ",\n"), tokens.args[1], __k);
      });
    }
  });
}

export function emit_keywords__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return emit_keywords_items__lto_1ba4622a(__caps, (s + "    // Keywords\n"), kws, __k);
    }
  });
}

export function emit_keywords_items__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const kw = kws.args[0];
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_384) => {
        return emit_keywords_items__lto_1ba4622a(__caps, ((__lto_self_393) => {
          return (__lto_self_393 + (((("    " + __lto_other_384) + ", // '") + kw) + "'\n"));
        })(s), kws.args[1], __k);
      });
    }
  });
}

export function emit_symbols__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const sym = syms.args[0];
    return emit_symbols_items__lto_1ba4622a(((syms.args[1][LUMO_TAG] === "nil") ? s : ((__match_223) => {
      return (s + "    // Symbols\n");
    })(syms)), syms);
  }
}

export function emit_symbols_items__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const sym = syms.args[0];
    const line = (((("    " + symbol_variant__lto_8227044e(sym)) + ", // '") + sym) + "'\n");
    return emit_symbols_items__lto_1ba4622a(((__lto_self_417) => {
      return (__lto_self_417 + line);
    })(s), syms.args[1]);
  }
}

export function emit_node_kinds__lto_1ba4622a(__caps, s, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = rules.args[1];
      const __match_226 = rules.args[0];
      const name = __match_226.args[0];
      const __match_227 = __match_226.args[1];
      if ((__match_227[LUMO_TAG] === "sequence")) {
        const elems = __match_227.args[0];
        return to_screaming_snake(__caps, name, (__lto_other_428) => {
          return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_437) => {
            return (__lto_self_437 + (((("    " + __lto_other_428) + ", // ") + name) + "\n"));
          })(s), rest, __k);
        });
      } else if ((__match_227[LUMO_TAG] === "alternatives")) {
        if (is_token_only_alternatives__lto_9309ae26(__match_227.args[0])) {
          return to_screaming_snake(__caps, name, (__lto_other_448) => {
            return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_457) => {
              return (__lto_self_457 + (((("    " + __lto_other_448) + ", // ") + name) + " (token wrapper)\n"));
            })(s), rest, __k);
          });
        } else {
          return emit_node_kinds__lto_1ba4622a(__caps, s, rest, __k);
        }
      } else {
        const atom_names = __match_227.args[0];
        return emit_pratt_alt_kinds(__caps, s, __match_227.args[1], (s2) => {
          return emit_node_kinds__lto_1ba4622a(__caps, s2, rest, __k);
        });
      }
    }
  });
}

export function emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, s, alts, seen, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_230 = alts.args[0];
      const name = __match_230.args[0];
      const elems = __match_230.args[1];
      const bp = __match_230.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, s, rest, seen, __k);
      } else {
        return to_screaming_snake(__caps, name, (__lto_other_468) => {
          return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, ((__lto_self_477) => {
            return (__lto_self_477 + (((("    " + __lto_other_468) + ", // ") + name) + "\n"));
          })(s), rest, List["cons"](name, seen), __k);
        });
      }
    }
  });
}

export function emit_from_keyword__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return emit_keyword_arms__lto_1ba4622a(__caps, ((s + "\n    pub fn from_keyword(text: &str) -> Option<Self> {\n") + "        match text {\n"), kws, (s) => {
        return __k((((s + "            _ => None,\n") + "        }\n") + "    }\n"));
      });
    }
  });
}

export function emit_keyword_arms__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const kw = kws.args[0];
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_504) => {
        return emit_keyword_arms__lto_1ba4622a(__caps, ((__lto_self_517) => {
          return (__lto_self_517 + (((("            \"" + kw) + "\" => Some(Self::") + __lto_other_504) + "),\n"));
        })(s), kws.args[1], __k);
      });
    }
  });
}

export function emit_from_symbol__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return (((emit_symbol_arms__lto_1ba4622a(((s + "\n    pub fn from_symbol(text: &str) -> Option<Self> {\n") + "        match text {\n"), syms) + "            _ => None,\n") + "        }\n") + "    }\n");
  }
}

export function emit_symbol_arms__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const sym = syms.args[0];
    const line = (((("            \"" + sym) + "\" => Some(Self::") + symbol_variant__lto_8227044e(sym)) + "),\n");
    return emit_symbol_arms__lto_1ba4622a(((__lto_self_557) => {
      return (__lto_self_557 + line);
    })(s), syms.args[1]);
  }
}

export function generate_ast__lto_1ba4622a(__caps, grammar, __k) {
  return __thunk(() => {
    const attrs = grammar.args[1];
    return emit_ast_rules(__caps, ((((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n\n") + "use super::SyntaxKind;\n") + "use super::{SyntaxNode, SyntaxElement, LosslessToken};\n\n") + "pub trait AstNode<'a>: Sized {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self>;\n") + "    fn syntax(&self) -> &'a SyntaxNode;\n") + "}\n\n"), grammar.args[0], grammar.args[2], __k);
  });
}

export function emit_enum_from_names__lto_1ba4622a(s, enum_name, names) {
  return (((emit_syntax_arms_from_names__lto_1ba4622a((((emit_cast_chain_from_names__lto_1ba4622a(((((((emit_variants_from_names__lto_1ba4622a((((s + "pub enum ") + enum_name) + "<'a> {\n"), names) + "}\n\n") + "impl<'a> AstNode<'a> for ") + enum_name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        None\n"), names) + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode {\n") + "        match self {\n"), names) + "        }\n") + "    }\n") + "}\n\n");
}

export function emit_variants_from_names__lto_1ba4622a(s, names) {
  if ((names[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = names.args[0];
    return emit_variants_from_names__lto_1ba4622a(((__lto_self_649) => {
      return (__lto_self_649 + "<'a>),\n");
    })(((__lto_self_651) => {
      return (__lto_self_651 + name);
    })(((__lto_self_653) => {
      return (__lto_self_653 + "(");
    })(((__lto_self_655) => {
      return (__lto_self_655 + name);
    })(((__lto_self_657) => {
      return (__lto_self_657 + "    ");
    })(s))))), names.args[1]);
  }
}

export function emit_cast_chain_from_names__lto_1ba4622a(s, names) {
  if ((names[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = names.args[0];
    return emit_cast_chain_from_names__lto_1ba4622a(((__lto_self_685) => {
      return (__lto_self_685 + (((("            .or_else(|| " + name) + "::cast(node).map(Self::") + name) + "))\n"));
    })(s), names.args[1]);
  }
}

export function emit_syntax_arms_from_names__lto_1ba4622a(s, names) {
  if ((names[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return emit_syntax_arms_from_names__lto_1ba4622a(((__lto_self_697) => {
      return (__lto_self_697 + (("            Self::" + names.args[0]) + "(n) => n.syntax(),\n"));
    })(s), names.args[1]);
  }
}

export function emit_struct_node__lto_1ba4622a(__caps, s, name, elems, token_defs, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return emit_accessors__lto_1ba4622a(__caps, (((((((((((((s + "pub struct ") + name) + "<'a>(pub(crate) &'a SyntaxNode);\n\n") + "impl<'a> AstNode<'a> for ") + name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        (node.kind == SyntaxKind::") + kind) + ").then(|| Self(node))\n") + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n") + "}\n\n"), name, elems, token_defs, __k);
  });
}

export function emit_accessors__lto_1ba4622a(__caps, s, struct_name, elems, token_defs, __k) {
  return __thunk(() => {
    if (has_labeled_elements(elems)) {
      return emit_accessors_for_elements(__caps, (((s + "impl<'a> ") + struct_name) + "<'a> {\n"), elems, token_defs, (s) => {
        return __k((s + "}\n\n"));
      });
    } else {
      return __k(s);
    }
  });
}

export function emit_token_accessor__lto_1ba4622a(__caps, s, label, t, repeated, __k) {
  return token_kind_from_ref(__caps, t, (kind) => {
    if (repeated) {
      return __k(((a, b) => {
        return (a + b);
      })((((((((((s + "    pub fn ") + label) + "(&self) -> impl Iterator<Item = &'a LosslessToken> + 'a {\n") + "        self.0.children.iter().filter_map(|c| match c {\n") + "            SyntaxElement::Token(t) if t.kind == SyntaxKind::") + kind) + " => Some(t),\n") + "            _ => None,\n") + "        })\n"), "    }\n"));
    } else {
      return __k(((a, b) => {
        return (a + b);
      })((((((((((s + "    pub fn ") + label) + "(&self) -> Option<&'a LosslessToken> {\n") + "        self.0.children.iter().find_map(|c| match c {\n") + "            SyntaxElement::Token(t) if t.kind == SyntaxKind::") + kind) + " => Some(t),\n") + "            _ => None,\n") + "        })\n"), "    }\n"));
    }
  });
}

export function emit_node_accessor__lto_1ba4622a(s, label, node_name, repeated) {
  if (repeated) {
    return ((((((((((((s + "    pub fn ") + label) + "(&self) -> impl Iterator<Item = ") + node_name) + "<'a>> + 'a {\n") + "        self.0.children.iter().filter_map(|c| match c {\n") + "            SyntaxElement::Node(n) => ") + node_name) + "::cast(n),\n") + "            _ => None,\n") + "        })\n") + "    }\n");
  } else {
    return ((((((((((((s + "    pub fn ") + label) + "(&self) -> Option<") + node_name) + "<'a>> {\n") + "        self.0.children.iter().find_map(|c| match c {\n") + "            SyntaxElement::Node(n) => ") + node_name) + "::cast(n),\n") + "            _ => None,\n") + "        })\n") + "    }\n");
  }
}

export function emit_enum_node__lto_1ba4622a(s, name, alts) {
  return (((emit_enum_syntax_arms__lto_1ba4622a((((emit_enum_cast_chain__lto_1ba4622a(((((((emit_enum_variants__lto_1ba4622a((((s + "pub enum ") + name) + "<'a> {\n"), alts) + "}\n\n") + "impl<'a> AstNode<'a> for ") + name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        None\n"), alts) + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode {\n") + "        match self {\n"), alts) + "        }\n") + "    }\n") + "}\n\n");
}

export function emit_enum_variants__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = alts.args[0].args[0];
    return emit_enum_variants__lto_1ba4622a(((__lto_self_1005) => {
      return (__lto_self_1005 + "<'a>),\n");
    })(((__lto_self_1007) => {
      return (__lto_self_1007 + name);
    })(((__lto_self_1009) => {
      return (__lto_self_1009 + "(");
    })(((__lto_self_1011) => {
      return (__lto_self_1011 + name);
    })(((__lto_self_1013) => {
      return (__lto_self_1013 + "    ");
    })(s))))), alts.args[1]);
  }
}

export function emit_enum_cast_chain__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = alts.args[0].args[0];
    return emit_enum_cast_chain__lto_1ba4622a(((__lto_self_1041) => {
      return (__lto_self_1041 + (((("            .or_else(|| " + name) + "::cast(node).map(Self::") + name) + "))\n"));
    })(s), alts.args[1]);
  }
}

export function emit_enum_syntax_arms__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return emit_enum_syntax_arms__lto_1ba4622a(((__lto_self_1053) => {
      return (__lto_self_1053 + (("            Self::" + alts.args[0].args[0]) + "(n) => n.syntax(),\n"));
    })(s), alts.args[1]);
  }
}

export function generate_parser__lto_1ba4622a(__caps, grammar, __k) {
  return __thunk(() => {
    const attrs = grammar.args[1];
    return emit_parser_impl__lto_1ba4622a(__caps, emit_parser_boilerplate__lto_1ba4622a((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n\n") + "use lumo_lexer::{lex_lossless, Keyword, LosslessTokenKind as LexKind};\n") + "use lumo_span::Span;\n\n") + "use crate::syntax_kind::SyntaxKind;\n\n")), grammar.args[0], grammar.args[2], (s) => {
      return __k(s);
    });
  });
}

export function emit_parser_boilerplate__lto_1ba4622a(s) {
  return emit_lexer_kind_map__lto_1ba4622a(((((((((((((((((((((((((((((((((((((((((s + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct ParseError {\n    pub span: Span,\n    pub message: String,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct LosslessToken {\n    pub kind: SyntaxKind,\n    pub span: Span,\n    pub text: String,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub enum SyntaxElement {\n    Node(Box<SyntaxNode>),\n    Token(LosslessToken),\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct SyntaxNode {\n    pub kind: SyntaxKind,\n    pub span: Span,\n    pub children: Vec<SyntaxElement>,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct ParseOutput {\n    pub root: SyntaxNode,\n    pub errors: Vec<ParseError>,\n}\n\n") + "pub fn parse(source: &str) -> ParseOutput {\n") + "    let lexed = lex_lossless(source);\n") + "    let mut p = Parser { tokens: lexed.tokens, index: 0, errors: Vec::new() };\n") + "    let root = p.parse_file();\n") + "    ParseOutput { root, errors: p.errors }\n}\n\n") + "pub fn node_text(node: &SyntaxNode) -> String {\n") + "    let mut out = String::new();\n") + "    write_node_text(node, &mut out);\n") + "    out\n}\n\n") + "fn write_node_text(node: &SyntaxNode, out: &mut String) {\n") + "    for child in &node.children {\n") + "        match child {\n") + "            SyntaxElement::Node(n) => write_node_text(n, out),\n") + "            SyntaxElement::Token(t) => out.push_str(&t.text),\n") + "        }\n    }\n}\n\n") + "fn node_from_children(kind: SyntaxKind, children: Vec<SyntaxElement>) -> SyntaxNode {\n") + "    let span = children_span(&children);\n") + "    SyntaxNode { kind, span, children }\n}\n\n") + "fn children_span(children: &[SyntaxElement]) -> Span {\n") + "    let start = children.iter().find_map(|c| match c {\n") + "        SyntaxElement::Token(t) => Some(t.span.start),\n") + "        SyntaxElement::Node(n) => if n.children.is_empty() { None } else { Some(n.span.start) },\n") + "    }).unwrap_or(0);\n") + "    let end = children.iter().rev().find_map(|c| match c {\n") + "        SyntaxElement::Token(t) => Some(t.span.end),\n") + "        SyntaxElement::Node(n) => if n.children.is_empty() { None } else { Some(n.span.end) },\n") + "    }).unwrap_or(0);\n") + "    Span::new(start, end)\n}\n\n") + "fn lexer_token_to_lst(t: lumo_lexer::LosslessToken) -> LosslessToken {\n") + "    LosslessToken { kind: lexer_kind_to_syntax_kind(&t.kind, &t.text), span: t.span, text: t.text }\n}\n\n"));
}

export function emit_lexer_kind_map__lto_1ba4622a(s) {
  return ((((((((((((((((((((((((((((s + "fn lexer_kind_to_syntax_kind(kind: &LexKind, text: &str) -> SyntaxKind {\n") + "    match kind {\n") + "        LexKind::Ident => SyntaxKind::IDENT,\n") + "        LexKind::StringLit => SyntaxKind::STRING_LIT,\n") + "        LexKind::NumberLit => SyntaxKind::NUMBER_LIT,\n") + "        LexKind::Whitespace => SyntaxKind::WHITESPACE,\n") + "        LexKind::Newline => SyntaxKind::NEWLINE,\n") + "        LexKind::Unknown => SyntaxKind::UNKNOWN,\n") + "        LexKind::Keyword(kw) => match kw {\n") + "            Keyword::Data => SyntaxKind::DATA_KW,\n") + "            Keyword::Fn => SyntaxKind::FN_KW,\n") + "            Keyword::Extern => SyntaxKind::EXTERN_KW,\n") + "            Keyword::Let => SyntaxKind::LET_KW,\n") + "            Keyword::In => SyntaxKind::IN_KW,\n") + "            Keyword::Thunk => SyntaxKind::THUNK_KW,\n") + "            Keyword::Force => SyntaxKind::FORCE_KW,\n") + "            Keyword::Match => SyntaxKind::MATCH_KW,\n") + "            Keyword::Cap => SyntaxKind::CAP_KW,\n") + "            Keyword::Handle => SyntaxKind::HANDLE_KW,\n") + "            Keyword::Bundle => SyntaxKind::BUNDLE_KW,\n") + "            Keyword::Use => SyntaxKind::USE_KW,\n") + "            Keyword::Impl => SyntaxKind::IMPL_KW,\n") + "            Keyword::If => SyntaxKind::IF_KW,\n") + "            Keyword::Else => SyntaxKind::ELSE_KW,\n") + "            _ => SyntaxKind::UNKNOWN,\n") + "        },\n") + "        LexKind::Symbol(_) => SyntaxKind::from_symbol(text).unwrap_or(SyntaxKind::UNKNOWN),\n") + "    }\n}\n\n");
}

export function emit_parser_struct__lto_1ba4622a(s) {
  return (((((s + "struct Parser {\n") + "    tokens: Vec<lumo_lexer::LosslessToken>,\n") + "    index: usize,\n") + "    errors: Vec<ParseError>,\n") + "}\n\n");
}

export function emit_parser_helpers__lto_1ba4622a(s) {
  return (((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((s + "impl Parser {\n") + "    fn eof(&self) -> bool { self.index >= self.tokens.len() }\n") + "    fn current(&self) -> Option<&lumo_lexer::LosslessToken> { self.tokens.get(self.index) }\n") + "    fn bump(&mut self) -> Option<LosslessToken> {\n") + "        let token = self.tokens.get(self.index).cloned();\n") + "        if token.is_some() { self.index += 1; }\n") + "        token.map(lexer_token_to_lst)\n") + "    }\n") + "    fn is_trivia_lex(kind: &LexKind) -> bool {\n") + "        matches!(kind, LexKind::Whitespace | LexKind::Newline)\n") + "    }\n") + "    fn at_trivia(&self) -> bool {\n") + "        self.current().map(|t| Self::is_trivia_lex(&t.kind)).unwrap_or(false)\n") + "    }\n") + "    fn skip_trivia_into(&mut self, children: &mut Vec<SyntaxElement>) {\n") + "        while self.at_trivia() { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "    }\n") + "    fn peek_non_trivia_token(&self, n: usize) -> Option<&lumo_lexer::LosslessToken> {\n") + "        let mut count = 0; let mut i = self.index;\n") + "        while i < self.tokens.len() {\n") + "            let tok = &self.tokens[i];\n") + "            if !Self::is_trivia_lex(&tok.kind) {\n") + "                if count == n { return Some(tok); }\n") + "                count += 1;\n") + "            }\n") + "            i += 1;\n") + "        }\n") + "        None\n") + "    }\n") + "    fn at_keyword(&self, kw: Keyword) -> bool {\n") + "        matches!(self.current().map(|t| &t.kind), Some(LexKind::Keyword(actual)) if *actual == kw)\n") + "    }\n") + "    fn at_non_trivia_keyword(&self, kw: Keyword) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::Keyword(actual)) if *actual == kw)\n") + "    }\n") + "    fn at_non_trivia_symbol(&self, text: &str) -> bool {\n") + "        self.peek_non_trivia_token(0).map(|t| t.text.as_str()) == Some(text)\n") + "    }\n") + "    fn at_non_trivia_ident(&self) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::Ident))\n") + "    }\n") + "    fn at_non_trivia_ident_text(&self, text: &str) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0), Some(tok) if matches!(tok.kind, LexKind::Ident) && tok.text == text)\n") + "    }\n") + "    fn at_ident(&self) -> bool {\n") + "        matches!(self.current().map(|t| &t.kind), Some(LexKind::Ident))\n") + "    }\n") + "    fn at_symbol_text(&self, text: &str) -> bool {\n") + "        self.current().map(|t| t.text.as_str()) == Some(text)\n") + "    }\n") + "    fn at_trivia_or_unknown(&self) -> bool {\n") + "        self.current().map(|t| matches!(t.kind, LexKind::Whitespace | LexKind::Newline | LexKind::Unknown)).unwrap_or(false)\n") + "    }\n") + "    fn error_here(&mut self, message: &str) {\n") + "        let span = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));\n") + "        self.errors.push(ParseError { span, message: message.to_owned() });\n") + "    }\n") + "    fn expect_keyword(&mut self, kw: Keyword, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_keyword(kw) { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(\"expected keyword\"); }\n") + "    }\n") + "    fn expect_symbol(&mut self, sym: &str, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_symbol_text(sym) { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(&format!(\"expected '{}'\", sym)); }\n") + "    }\n") + "    fn expect_ident(&mut self, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_ident() { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(\"expected identifier\"); }\n") + "    }\n") + "}\n\n");
}

export function emit_parser_impl__lto_1ba4622a(__caps, s, token_defs, rules, __k) {
  return __thunk(() => {
    return emit_parse_rules(__caps, (emit_parser_helpers__lto_1ba4622a(emit_parser_struct__lto_1ba4622a(s)) + "impl Parser {\n"), token_defs, rules, (s) => {
      return __k(((a, b) => {
        return (a + b);
      })(s, "}\n"));
    });
  });
}

export function emit_can_parse_method__lto_1ba4622a(__caps, s, name, body, token_defs, __k) {
  return __thunk(() => {
    return to_snake(__caps, name, (__lto_other_1666) => {
      return make_body_lookahead(__caps, body, token_defs, (cond) => {
        return __k(((a, b) => {
          return (a + b);
        })(((((s + "    fn ") + ("can_parse_" + __lto_other_1666)) + "(&self) -> bool { ") + cond), " }\n"));
      });
    });
  });
}

export function make_alts_lookahead__lto_1ba4622a(__caps, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const rest = alts.args[1];
      return to_snake(__caps, alts.args[0].args[0], (__lto_other_1692) => {
        const cond = (("self.can_parse_" + __lto_other_1692) + "()");
        if ((rest[LUMO_TAG] === "nil")) {
          return __k(cond);
        } else {
          return make_alts_lookahead__lto_1ba4622a(__caps, rest, (__lto_other_1698) => {
            return __k(((a, b) => {
              return (a + b);
            })((cond + " || "), __lto_other_1698));
          });
        }
      });
    }
  });
}

export function make_pratt_lookahead__lto_8227044e(__caps, atom_names, alts, __k) {
  return make_prefix_alts_lookahead__lto_8227044e(__caps, alts, (prefix_cond) => {
    return make_atoms_lookahead__lto_1ba4622a(__caps, atom_names, (atom_cond) => {
      if ((prefix_cond === "")) {
        return __k(atom_cond);
      } else if ((atom_cond === "false")) {
        return __k(prefix_cond);
      } else {
        return __k(((a, b) => {
          return (a + b);
        })((prefix_cond + " || "), atom_cond));
      }
    });
  });
}

export function make_prefix_alts_lookahead__lto_8227044e(__caps, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k("");
    } else {
      const rest = alts.args[1];
      const __match_256 = alts.args[0];
      const elems = __match_256.args[1];
      const __match_257 = __match_256.args[2];
      const rbp = __match_257.args[1];
      if ((__match_257.args[0][LUMO_TAG] === "num")) {
        return make_prefix_alts_lookahead__lto_8227044e(__caps, rest, __k);
      } else {
        return to_snake(__caps, __match_256.args[0], (__lto_other_1724) => {
          const cond = (("self.at_pratt_" + __lto_other_1724) + "()");
          return make_prefix_alts_lookahead__lto_8227044e(__caps, rest, (rest_cond) => {
            if ((rest_cond === "")) {
              return __k(cond);
            } else {
              return __k(((a, b) => {
                return (a + b);
              })((cond + " || "), rest_cond));
            }
          });
        });
      }
    }
  });
}

export function make_atoms_lookahead__lto_1ba4622a(__caps, atom_names, __k) {
  return __thunk(() => {
    if ((atom_names[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const rest = atom_names.args[1];
      return to_snake(__caps, atom_names.args[0], (__lto_other_1744) => {
        const cond = (("self.can_parse_" + __lto_other_1744) + "()");
        if ((rest[LUMO_TAG] === "nil")) {
          return __k(cond);
        } else {
          return make_atoms_lookahead__lto_1ba4622a(__caps, rest, (__lto_other_1750) => {
            return __k(((a, b) => {
              return (a + b);
            })((cond + " || "), __lto_other_1750));
          });
        }
      });
    }
  });
}

export function emit_parse_sequence_rule__lto_8227044e(__caps, s, name, elems, token_defs, __k) {
  return __thunk(() => {
    if ((name === "File")) {
      return __k(emit_parse_file_with_recovery__lto_1ba4622a(s));
    } else {
      return to_snake(__caps, name, (__lto_other_1762) => {
        return to_screaming_snake(__caps, name, (kind) => {
          return emit_parse_elements(__caps, ((((s + "    fn ") + ("parse_" + __lto_other_1762)) + "(&mut self) -> SyntaxNode {\n") + "        let mut children = Vec::new();\n"), elems, token_defs, "        ", (s) => {
            return __k(((a, b) => {
              return (a + b);
            })((((s + "        node_from_children(SyntaxKind::") + kind) + ", children)\n"), "    }\n\n"));
          });
        });
      });
    }
  });
}

export function emit_parse_file_with_recovery__lto_1ba4622a(s) {
  return ((((((((((((((s + "    fn parse_file(&mut self) -> SyntaxNode {\n") + "        let mut children = Vec::new();\n") + "        while !self.eof() {\n") + "            self.skip_trivia_into(&mut children);\n") + "            if self.eof() { break; }\n") + "            if self.can_parse_item() {\n") + "                children.push(SyntaxElement::Node(Box::new(self.parse_item())));\n") + "            } else {\n") + "                children.push(SyntaxElement::Node(Box::new(\n") + "                    node_from_children(SyntaxKind::ERROR, vec![SyntaxElement::Token(self.bump().unwrap())])\n") + "                )));\n") + "            }\n") + "        }\n") + "        node_from_children(SyntaxKind::FILE, children)\n    }\n\n");
}

export function emit_parse_alt_rule__lto_1ba4622a(__caps, s, name, alts, __k) {
  return __thunk(() => {
    return to_snake(__caps, name, (__lto_other_1854) => {
      return emit_alt_dispatch__lto_1ba4622a(__caps, (((s + "    fn ") + ("parse_" + __lto_other_1854)) + "(&mut self) -> SyntaxNode {\n"), alts, "        ", (s) => {
        return __k(((a, b) => {
          return (a + b);
        })(((((s + "        self.error_here(\"expected ") + name) + "\");\n") + "        node_from_children(SyntaxKind::ERROR, Vec::new())\n"), "    }\n\n"));
      });
    });
  });
}

export function emit_alt_dispatch__lto_1ba4622a(__caps, s, alts, indent, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const alt_name = alts.args[0].args[0];
      return to_snake(__caps, alt_name, (__lto___lto_other_3271_3297) => {
        return to_snake(__caps, alt_name, (__lto_other_1892) => {
          return emit_alt_dispatch__lto_1ba4622a(__caps, ((((((s + indent) + "if ") + (("self.can_parse_" + __lto___lto_other_3271_3297) + "()")) + " { return ") + (("self.parse_" + __lto_other_1892) + "()")) + "; }\n"), alts.args[1], indent, __k);
        });
      });
    }
  });
}

export function emit_parse_pratt_rule__lto_1ba4622a(__caps, s, name, atom_names, alts, token_defs, __k) {
  return __thunk(() => {
    return to_snake(__caps, name, (__lto_other_1922) => {
      const fn_name = ("parse_" + __lto_other_1922);
      return emit_pratt_loop__lto_1ba4622a(__caps, (((((((((((((s + "    fn ") + fn_name) + "(&mut self) -> SyntaxNode {\n") + "        self.") + fn_name) + "_bp(0)\n") + "    }\n\n") + "    fn ") + fn_name) + "_bp(&mut self, min_bp: u8) -> SyntaxNode {\n") + "        let mut lhs = self.") + fn_name) + "_atom();\n"), name, alts, token_defs, "        ", (s) => {
        return emit_pratt_atom_dispatch__lto_1ba4622a(__caps, (((((s + "        lhs\n") + "    }\n\n") + "    fn ") + fn_name) + "_atom(&mut self) -> SyntaxNode {\n"), name, atom_names, alts, token_defs, "        ", (s) => {
          return emit_pratt_at_predicates(__caps, (s + "    }\n\n"), alts, token_defs, __k);
        });
      });
    });
  });
}

export function emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, s, alts, token_defs, seen, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_266 = alts.args[0];
      const name = __match_266.args[0];
      const bp = __match_266.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, s, rest, token_defs, seen, __k);
      } else {
        return to_snake(__caps, name, (__lto_other_2002) => {
          return make_first_elem_lookahead(__caps, __match_266.args[1], token_defs, (cond) => {
            return emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, (((((s + "    fn ") + ("at_pratt_" + __lto_other_2002)) + "(&self) -> bool { ") + cond) + " }\n"), rest, token_defs, List["cons"](name, seen), __k);
          });
        });
      }
    }
  });
}

export function emit_pratt_loop__lto_1ba4622a(__caps, s, rule_name, alts, token_defs, indent, __k) {
  return __thunk(() => {
    return to_snake(__caps, rule_name, (__lto_other_2026) => {
      return emit_pratt_infix_alts__lto_1ba4622a(__caps, ((s + indent) + "loop {\n"), ("parse_" + __lto_other_2026), alts, token_defs, ((__lto_self_2037) => {
        return (__lto_self_2037 + "    ");
      })(indent), (s) => {
        return __k(((a, b) => {
          return (a + b);
        })((((s + indent) + "    break;\n") + indent), "}\n"));
      });
    });
  });
}

export function emit_pratt_infix_alts__lto_1ba4622a(__caps, s, fn_name, alts, token_defs, indent, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_269 = alts.args[0];
      const name = __match_269.args[0];
      const __match_270 = __match_269.args[2];
      const __match_271 = __match_270.args[0];
      if ((__match_271[LUMO_TAG] === "none")) {
        return emit_pratt_infix_alts__lto_1ba4622a(__caps, s, fn_name, rest, token_defs, indent, __k);
      } else {
        return to_screaming_snake(__caps, name, (kind) => {
          const lbp_str = Number.to_string(__match_271.args[0]);
          return to_snake(__caps, name, (__lto_other_2088) => {
            const __k_121 = (rbp_str) => {
              return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((((((((((s + indent) + "// ") + name) + " (lbp=") + lbp_str) + ")\n") + indent) + "if self.at_pratt_") + __lto_other_2088) + "() && ") + lbp_str) + " > min_bp {\n") + indent) + "    let mut children = vec![SyntaxElement::Node(Box::new(lhs))];\n"), __match_269.args[1], token_defs, fn_name, rbp_str, ((__lto_self_2113) => {
                return (__lto_self_2113 + "    ");
              })(indent), (sd) => {
                return emit_pratt_infix_alts__lto_1ba4622a(__caps, ((((((((sd + indent) + "    lhs = node_from_children(SyntaxKind::") + kind) + ", children);\n") + indent) + "    continue;\n") + indent) + "}\n"), fn_name, rest, token_defs, indent, __k);
              });
            };
            const __match_272 = __match_270.args[1];
            if ((__match_272[LUMO_TAG] === "none")) {
              return __k_121(lbp_str);
            } else {
              return __k_121(Number.to_string(__match_272.args[0]));
            }
          });
        });
      }
    }
  });
}

export function emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, elems, token_defs, fn_name, rbp_str, indent, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = elems.args[1];
      const __match_274 = elems.args[0];
      if ((__match_274[LUMO_TAG] === "node")) {
        const rname = __match_274.args[0].args[0];
        if ((rname === "Expr")) {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.") + fn_name) + "_bp(") + rbp_str) + "))));\n"), rest, token_defs, fn_name, rbp_str, indent, __k);
        } else {
          return to_snake(__caps, rname, (__lto_other_2180) => {
            return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.parse_") + __lto_other_2180) + "())));\n"), rest, token_defs, fn_name, rbp_str, indent, __k);
          });
        }
      } else if ((__match_274[LUMO_TAG] === "labeled")) {
        const label = __match_274.args[0];
        return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](__match_274.args[1], rest), token_defs, fn_name, rbp_str, indent, __k);
      } else {
        return ((__match_274[LUMO_TAG] === "token") ? ((t) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, emit_parse_token_element__lto_1ba4622a(s, t, indent), rest, token_defs, fn_name, rbp_str, indent, __k);
        })(__match_274.args[0]) : ((__match_274[LUMO_TAG] === "optional") ? ((inner) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](inner, rest), token_defs, fn_name, rbp_str, indent, __k);
        })(__match_274.args[0]) : ((__match_274[LUMO_TAG] === "repeated") ? ((inner) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](inner, rest), token_defs, fn_name, rbp_str, indent, __k);
        })(__match_274.args[0]) : ((gelems) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, list_concat_elem(gelems, rest), token_defs, fn_name, rbp_str, indent, __k);
        })(__match_274.args[0]))));
      }
    }
  });
}

export function emit_pratt_atom_dispatch__lto_1ba4622a(__caps, s, rule_name, atom_names, alts, token_defs, indent, __k) {
  return emit_prefix_alts__lto_1ba4622a(__caps, s, rule_name, alts, token_defs, indent, (s2) => {
    return emit_atom_dispatch_alts__lto_1ba4622a(__caps, s2, atom_names, indent, (s2) => {
      return __k(((a, b) => {
        return (a + b);
      })((((s2 + indent) + "self.error_here(\"expected expression\");\n") + indent), "node_from_children(SyntaxKind::ERROR, Vec::new())\n"));
    });
  });
}

export function emit_prefix_alts__lto_1ba4622a(__caps, s, rule_name, alts, token_defs, indent, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_278 = alts.args[0];
      const name = __match_278.args[0];
      const __match_279 = __match_278.args[2];
      if ((__match_279.args[0][LUMO_TAG] === "num")) {
        return emit_prefix_alts__lto_1ba4622a(__caps, s, rule_name, rest, token_defs, indent, __k);
      } else {
        return to_screaming_snake(__caps, name, (kind) => {
          const __k_130 = (rbp_str) => {
            return to_snake(__caps, rule_name, (__lto_other_2210) => {
              return to_snake(__caps, name, (__lto_other_2216) => {
                return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((s + indent) + "if self.at_pratt_") + __lto_other_2216) + "() {\n") + indent) + "    let mut children = Vec::new();\n"), __match_278.args[1], token_defs, ("parse_" + __lto_other_2210), rbp_str, ((__lto_self_2237) => {
                  return (__lto_self_2237 + "    ");
                })(indent), (sc) => {
                  return emit_prefix_alts__lto_1ba4622a(__caps, ((((((sc + indent) + "    return node_from_children(SyntaxKind::") + kind) + ", children);\n") + indent) + "}\n"), rule_name, rest, token_defs, indent, __k);
                });
              });
            });
          };
          const __match_281 = __match_279.args[1];
          if ((__match_281[LUMO_TAG] === "none")) {
            return __k_130("0");
          } else {
            return __k_130(Number.to_string(__match_281.args[0]));
          }
        });
      }
    }
  });
}

export function emit_atom_dispatch_alts__lto_1ba4622a(__caps, s, atom_names, indent, __k) {
  return __thunk(() => {
    if ((atom_names[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const name = atom_names.args[0];
      return to_snake(__caps, name, (__lto_other_2268) => {
        return to_snake(__caps, name, (__lto_other_2276) => {
          return emit_atom_dispatch_alts__lto_1ba4622a(__caps, ((((((s + indent) + "if ") + (("self.can_parse_" + __lto_other_2268) + "()")) + " { ") + (("return self.parse_" + __lto_other_2276) + "();")) + " }\n"), atom_names.args[1], indent, __k);
        });
      });
    }
  });
}

export function emit_parse_element__lto_1ba4622a(__caps, s, elem, token_defs, indent, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      return __k(emit_parse_token_element__lto_1ba4622a(s, elem.args[0], indent));
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return __k(((a, b) => {
          return (a + b);
        })((((((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if matches!(self.current().map(|t| &t.kind), Some(LexKind::") + to_named_lex__lto_3890158f(name)) + ")) {\n") + indent) + "    children.push(SyntaxElement::Token(self.bump().unwrap()));\n") + indent) + "} else { self.error_here(\"expected ") + name), "\"); }\n"));
      } else {
        return to_snake(__caps, name, (__lto_other_2356) => {
          return __k(((a, b) => {
            return (a + b);
          })((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.parse_") + __lto_other_2356), "())));\n"));
        });
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        return emit_parse_element__lto_1ba4622a(__caps, s, elem.args[1], token_defs, indent, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_parse_optional__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_parse_repeated__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k);
      })(elem.args[0]) : ((gelems) => {
        return emit_parse_elements(__caps, s, gelems, token_defs, indent, __k);
      })(elem.args[0]))));
    }
  });
}

export function emit_parse_token_element__lto_1ba4622a(s, t, indent) {
  if ((t[LUMO_TAG] === "keyword")) {
    const kw = t.args[0];
    if (is_lexer_keyword__lto_3890158f(kw)) {
      return ((((s + indent) + "self.expect_keyword(Keyword::") + keyword_variant_pascal__lto_3890158f(kw)) + ", &mut children);\n");
    } else {
      return ((((((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if self.current().map(|t| matches!(t.kind, LexKind::Ident) && t.text.as_str() == \"") + kw) + "\").unwrap_or(false) {\n") + indent) + "    children.push(SyntaxElement::Token(self.bump().unwrap()));\n") + indent) + "} else { self.error_here(\"expected '") + kw) + "'\"); }\n");
    }
  } else if ((t[LUMO_TAG] === "symbol")) {
    return ((((s + indent) + "self.expect_symbol(\"") + t.args[0]) + "\", &mut children);\n");
  } else {
    const name = t.args[0];
    return ((((((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if matches!(self.current().map(|t| &t.kind), Some(LexKind::") + to_named_lex__lto_3890158f(name)) + ")) {\n") + indent) + "    children.push(SyntaxElement::Token(self.bump().unwrap()));\n") + indent) + "} else { self.error_here(\"expected ") + name) + "\"); }\n");
  }
}

export function emit_parse_optional__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k) {
  return make_element_lookahead__lto_1ba4622a(__caps, inner, token_defs, (cond) => {
    return emit_parse_element__lto_1ba4622a(__caps, ((((s + indent) + "if ") + cond) + " {\n"), inner, token_defs, ((__lto_self_2513) => {
      return (__lto_self_2513 + "    ");
    })(indent), (s2) => {
      return __k(((a, b) => {
        return (a + b);
      })((s2 + indent), "}\n"));
    });
  });
}

export function emit_parse_repeated__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k) {
  return make_element_lookahead__lto_1ba4622a(__caps, inner, token_defs, (cond) => {
    return emit_parse_element__lto_1ba4622a(__caps, ((((s + indent) + "while ") + cond) + " {\n"), inner, token_defs, ((__lto_self_2541) => {
      return (__lto_self_2541 + "    ");
    })(indent), (s2) => {
      return __k(((a, b) => {
        return (a + b);
      })((s2 + indent), "}\n"));
    });
  });
}

export function make_element_lookahead__lto_1ba4622a(__caps, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const __match_291 = elem.args[0];
      if ((__match_291[LUMO_TAG] === "keyword")) {
        const kw = __match_291.args[0];
        if (is_lexer_keyword__lto_3890158f(kw)) {
          return __k(((a, b) => {
            return (a + b);
          })(("self.at_non_trivia_keyword(Keyword::" + keyword_variant_pascal__lto_3890158f(kw)), ")"));
        } else {
          return __k(((a, b) => {
            return (a + b);
          })(("self.at_non_trivia_ident_text(\"" + kw), "\")"));
        }
      } else if ((__match_291[LUMO_TAG] === "symbol")) {
        return __k(((a, b) => {
          return (a + b);
        })(("self.at_non_trivia_symbol(\"" + __match_291.args[0]), "\")"));
      } else {
        const name = __match_291.args[0];
        return __k("self.at_non_trivia_ident()");
      }
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return __k("self.at_non_trivia_ident()");
      } else {
        return to_snake(__caps, name, (__lto_other_2580) => {
          return __k(((a, b) => {
            return (a + b);
          })(("self.can_parse_" + __lto_other_2580), "()"));
        });
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        return make_element_lookahead__lto_1ba4622a(__caps, elem.args[1], token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return make_element_lookahead__lto_1ba4622a(__caps, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return make_element_lookahead__lto_1ba4622a(__caps, inner, token_defs, __k);
      })(elem.args[0]) : ((gelems) => {
        return make_group_lookahead(__caps, gelems, token_defs, __k);
      })(elem.args[0]))));
    }
  });
}

export function to_named_lex__lto_3890158f(name) {
  if ((name === "Ident")) {
    return "Ident";
  } else if ((name === "StringLit")) {
    return "StringLit";
  } else if ((name === "NumberLit")) {
    return "NumberLit";
  } else {
    return "Ident";
  }
}

export function is_lexer_keyword__lto_3890158f(kw) {
  if ((kw === "data")) {
    return true;
  } else if ((kw === "fn")) {
    return true;
  } else if ((kw === "extern")) {
    return true;
  } else if ((kw === "let")) {
    return true;
  } else if ((kw === "in")) {
    return true;
  } else if ((kw === "thunk")) {
    return true;
  } else if ((kw === "force")) {
    return true;
  } else if ((kw === "match")) {
    return true;
  } else if ((kw === "cap")) {
    return true;
  } else if ((kw === "handle")) {
    return true;
  } else if ((kw === "bundle")) {
    return true;
  } else if ((kw === "use")) {
    return true;
  } else if ((kw === "impl")) {
    return true;
  } else if ((kw === "if")) {
    return true;
  } else if ((kw === "else")) {
    return true;
  } else {
    return false;
  }
}

export function keyword_variant_pascal__lto_3890158f(kw) {
  if ((kw === "data")) {
    return "Data";
  } else if ((kw === "fn")) {
    return "Fn";
  } else if ((kw === "extern")) {
    return "Extern";
  } else if ((kw === "let")) {
    return "Let";
  } else if ((kw === "in")) {
    return "In";
  } else if ((kw === "thunk")) {
    return "Thunk";
  } else if ((kw === "force")) {
    return "Force";
  } else if ((kw === "match")) {
    return "Match";
  } else if ((kw === "cap")) {
    return "Cap";
  } else if ((kw === "handle")) {
    return "Handle";
  } else if ((kw === "bundle")) {
    return "Bundle";
  } else if ((kw === "use")) {
    return "Use";
  } else if ((kw === "impl")) {
    return "Impl";
  } else if ((kw === "if")) {
    return "If";
  } else if ((kw === "else")) {
    return "Else";
  } else {
    return kw;
  }
}

export function to_snake_loop__lto_1fab3ad0(name, i, acc) {
  const __lto_b_2720 = String.len(name);
  const __match_328 = ((i < __lto_b_2720) ? Ordering["less"] : ((__match_327) => {
    if (__match_327) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_2720)));
  if (((__match_328[LUMO_TAG] === "less") ? false : ((__match_328[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __match_337 = ((code < 65) ? Ordering["less"] : ((__match_336) => {
      if (__match_336) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_337[LUMO_TAG] === "less") ? false : ((__match_337[LUMO_TAG] === "equal") ? true : true)) ? ((__match_341) => {
      if ((__match_341[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_341[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_2725) => {
      const __lto_other_2726 = 90;
      const __match_339 = (__lto_self_2725 < __lto_other_2726);
      if (__match_339) {
        return Ordering["less"];
      } else {
        const __match_340 = (__lto_self_2725 === __lto_other_2726);
        if (__match_340) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      let __match_334;
      let __match_333;
      if ((0 < i)) {
        __match_333 = Ordering["less"];
      } else if ((0 === i)) {
        __match_333 = Ordering["equal"];
      } else {
        __match_333 = Ordering["greater"];
      }
      if ((__match_333[LUMO_TAG] === "less")) {
        __match_334 = true;
      } else if ((__match_333[LUMO_TAG] === "equal")) {
        __match_334 = false;
      } else {
        __match_334 = false;
      }
      if (__match_334) {
        return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_2733) => {
          return (__lto_self_2733 + 1);
        })(i), ((__lto_self_2737) => {
          return (__lto_self_2737 + to_lower_char__lto_56361231(c));
        })(((__lto_self_2739) => {
          return (__lto_self_2739 + "_");
        })(acc)));
      } else {
        return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_2745) => {
          return (__lto_self_2745 + 1);
        })(i), ((__lto_self_2749) => {
          return (__lto_self_2749 + to_lower_char__lto_56361231(c));
        })(acc));
      }
    } else {
      return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_2753) => {
        return (__lto_self_2753 + 1);
      })(i), ((__lto_self_2757) => {
        return (__lto_self_2757 + c);
      })(acc));
    }
  }
}

export function to_lower_char__lto_56361231(c) {
  const code = String.char_code_at(c, 0);
  const __match_344 = ((code < 65) ? Ordering["less"] : ((__match_343) => {
    if (__match_343) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 65)));
  if (((__match_344[LUMO_TAG] === "less") ? false : ((__match_344[LUMO_TAG] === "equal") ? true : true))) {
    let __match_349;
    let __match_348;
    if ((code < 90)) {
      __match_348 = Ordering["less"];
    } else if ((code === 90)) {
      __match_348 = Ordering["equal"];
    } else {
      __match_348 = Ordering["greater"];
    }
    if ((__match_348[LUMO_TAG] === "less")) {
      __match_349 = true;
    } else if ((__match_348[LUMO_TAG] === "equal")) {
      __match_349 = true;
    } else {
      __match_349 = false;
    }
    if (__match_349) {
      return fromCharCode((code + 32));
    } else {
      return c;
    }
  } else {
    return c;
  }
}

export function emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return __k((((((((((((((s + "pub struct ") + name) + "<'a>(pub(crate) &'a SyntaxNode);\n\n") + "impl<'a> AstNode<'a> for ") + name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        (node.kind == SyntaxKind::") + kind) + ").then(|| Self(node))\n") + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n") + "}\n\n"));
  });
}

export function run__lto_3829b133(__caps, __k) {
  return __thunk(() => {
    const __lto_a_2828 = (__argv_length_raw() - 1);
    const __match_355 = ((__lto_a_2828 < 2) ? Ordering["less"] : ((__match_354) => {
      if (__match_354) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_a_2828, 2)));
    if (((__match_355[LUMO_TAG] === "less") ? true : ((__match_355[LUMO_TAG] === "equal") ? false : false))) {
      const __lto__err_2831 = __console_error("Usage: langue <input.langue> [output_dir]");
      return __k(__exit_process(1));
    } else {
      const file = __argv_at_raw(((__lto___lto_self_3278_3309) => {
        return (__lto___lto_self_3278_3309 + 1);
      })(1));
      return parse_grammar(__caps, readFileSync(file, "utf8"), (__cps_v_45) => {
        if ((__cps_v_45[LUMO_TAG] === "ok")) {
          return resolve_grammar(__caps, __cps_v_45.args[0], (grammar) => {
            const tokens = grammar.args[0];
            const count = list_length_rules__lto_92991de6(grammar.args[2]);
            return generate_syntax_kind__lto_1ba4622a(__caps, grammar, (syntax_kind_code) => {
              return generate_ast__lto_1ba4622a(__caps, grammar, (ast_code) => {
                return run_generate__lto_35421161(__caps, file, count, syntax_kind_code, ast_code, grammar, find_parser_path(grammar.args[1]), __k);
              });
            });
          });
        } else {
          const __lto__err_2835 = __console_error(((("Parse error at position " + Number.to_string(__cps_v_45.args[1])) + ": ") + __cps_v_45.args[0]));
          return __k(__exit_process(1));
        }
      });
    }
  });
}

export function run_generate__lto_35421161(__caps, file, count, syntax_kind_code, ast_code, grammar, parser_path, __k) {
  return __thunk(() => {
    const __lto_a_2850 = (__argv_length_raw() - 1);
    const __match_359 = ((__lto_a_2850 < 3) ? Ordering["less"] : ((__match_358) => {
      if (__match_358) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_a_2850, 3)));
    if (((__match_359[LUMO_TAG] === "less") ? true : ((__match_359[LUMO_TAG] === "equal") ? false : false))) {
      return write_output__lto_b8d7a8c4(__caps, ".", file, count, syntax_kind_code, ast_code, grammar, parser_path, __k);
    } else {
      return write_output__lto_b8d7a8c4(__caps, __argv_at_raw(((__lto___lto_self_3278_3318) => {
        return (__lto___lto_self_3278_3318 + 1);
      })(2)), file, count, syntax_kind_code, ast_code, grammar, parser_path, __k);
    }
  });
}

export function write_output__lto_b8d7a8c4(__caps, out_dir, file, count, syntax_kind_code, ast_code, grammar, parser_path, __k) {
  return __thunk(() => {
    const sk_path = (out_dir + "/syntax_kind.rs");
    const ast_path = (out_dir + "/ast.rs");
    const w1 = writeFileSync(sk_path, syntax_kind_code, "utf8");
    const w2 = writeFileSync(ast_path, ast_code, "utf8");
    const p1 = globalThis.console.log(((("Parsed " + Number.to_string(count)) + " rules from ") + file));
    const p2 = globalThis.console.log(("Wrote " + sk_path));
    const p3 = globalThis.console.log(("Wrote " + ast_path));
    if ((parser_path === "")) {
      return __k(((msg) => {
        return globalThis.console.log(msg);
      })(""));
    } else {
      const full_path = ((out_dir + "/") + parser_path);
      return generate_parser__lto_1ba4622a(__caps, grammar, (parser_code) => {
        const w3 = writeFileSync(full_path, parser_code, "utf8");
        return __k(((msg) => {
          return globalThis.console.log(msg);
        })(("Wrote " + full_path)));
      });
    }
  });
}

export function list_length_rules__lto_92991de6(xs) {
  if ((xs[LUMO_TAG] === "nil")) {
    return 0;
  } else {
    return (1 + list_length_rules__lto_92991de6(xs.args[1]));
  }
}

export function is_whitespace__lto_3890158f(c) {
  if ((c === " ")) {
    return true;
  } else if ((c === "\n")) {
    return true;
  } else if ((c === "\t")) {
    return true;
  } else if ((c === "\r")) {
    return true;
  } else {
    return false;
  }
}

export function is_alpha__lto_9309ae26(c) {
  const code = String.char_code_at(c, 0);
  const __match_368 = ((code < 97) ? Ordering["less"] : ((__match_367) => {
    if (__match_367) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_368[LUMO_TAG] === "less") ? false : ((__match_368[LUMO_TAG] === "equal") ? true : true))) {
    let __match_379;
    if ((code < 122)) {
      __match_379 = Ordering["less"];
    } else if ((code === 122)) {
      __match_379 = Ordering["equal"];
    } else {
      __match_379 = Ordering["greater"];
    }
    if ((__match_379[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_379[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return false;
    }
  } else {
    let __match_373;
    let __match_372;
    if ((code < 65)) {
      __match_372 = Ordering["less"];
    } else if ((code === 65)) {
      __match_372 = Ordering["equal"];
    } else {
      __match_372 = Ordering["greater"];
    }
    if ((__match_372[LUMO_TAG] === "less")) {
      __match_373 = false;
    } else if ((__match_372[LUMO_TAG] === "equal")) {
      __match_373 = true;
    } else {
      __match_373 = true;
    }
    if (__match_373) {
      let __match_376;
      if ((code < 90)) {
        __match_376 = Ordering["less"];
      } else if ((code === 90)) {
        __match_376 = Ordering["equal"];
      } else {
        __match_376 = Ordering["greater"];
      }
      if ((__match_376[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_376[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    } else {
      return false;
    }
  }
}

export function is_ident_continue__lto_3890158f(c) {
  if (is_alpha__lto_9309ae26(c)) {
    return true;
  } else if ((c === "_")) {
    return true;
  } else {
    return false;
  }
}

export function is_digit__lto_9309ae26(c) {
  const code = String.char_code_at(c, 0);
  const __match_384 = ((code < 48) ? Ordering["less"] : ((__match_383) => {
    if (__match_383) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 48)));
  if (((__match_384[LUMO_TAG] === "less") ? false : ((__match_384[LUMO_TAG] === "equal") ? true : true))) {
    let __match_388;
    if ((code < 57)) {
      __match_388 = Ordering["less"];
    } else if ((code === 57)) {
      __match_388 = Ordering["equal"];
    } else {
      __match_388 = Ordering["greater"];
    }
    if ((__match_388[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_388[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return false;
    }
  } else {
    return false;
  }
}

export function state_eof__lto_9309ae26(st) {
  const __lto_a_2958 = st.args[1];
  const __lto_b_2959 = String.len(st.args[0]);
  const __match_392 = ((__lto_a_2958 < __lto_b_2959) ? Ordering["less"] : ((__match_391) => {
    if (__match_391) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_2958, __lto_b_2959)));
  if ((__match_392[LUMO_TAG] === "less")) {
    return false;
  } else if ((__match_392[LUMO_TAG] === "equal")) {
    return true;
  } else {
    return true;
  }
}

export function state_peek__lto_9309ae26(st) {
  const src = st.args[0];
  const pos = st.args[1];
  const __lto_b_2963 = String.len(src);
  const __match_396 = ((pos < __lto_b_2963) ? Ordering["less"] : ((__match_395) => {
    if (__match_395) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(pos, __lto_b_2963)));
  if (((__match_396[LUMO_TAG] === "less") ? true : ((__match_396[LUMO_TAG] === "equal") ? false : false))) {
    return String.char_at(src, pos);
  } else {
    return "";
  }
}

export function state_advance__lto_92991de6(st, n) {
  return ParseState["mk"](st.args[0], ((__lto_self_2964) => {
    return (__lto_self_2964 + n);
  })(st.args[1]));
}

export function skip_ws__lto_1bb67705(st) {
  if (state_eof__lto_9309ae26(st)) {
    return st;
  } else {
    const c = state_peek__lto_9309ae26(st);
    if (is_whitespace__lto_3890158f(c)) {
      return skip_ws__lto_1bb67705(state_advance__lto_92991de6(st, 1));
    } else if ((c === "/")) {
      const next_pos = (state_pos(st) + 1);
      const __lto_b_2979 = String.len(state_src(st));
      const __match_404 = ((next_pos < __lto_b_2979) ? Ordering["less"] : ((__match_403) => {
        if (__match_403) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(next_pos, __lto_b_2979)));
      if (((__match_404[LUMO_TAG] === "less") ? true : ((__match_404[LUMO_TAG] === "equal") ? false : false))) {
        if ((String.char_at(state_src(st), next_pos) === "/")) {
          return skip_ws__lto_1bb67705(skip_line__lto_3890158f(state_advance__lto_92991de6(st, 2)));
        } else {
          return st;
        }
      } else {
        return st;
      }
    } else {
      return st;
    }
  }
}

export function skip_line__lto_3890158f(st) {
  if (state_eof__lto_9309ae26(st)) {
    return st;
  } else if ((state_peek__lto_9309ae26(st) === "\n")) {
    return state_advance__lto_92991de6(st, 1);
  } else {
    return skip_line__lto_3890158f(state_advance__lto_92991de6(st, 1));
  }
}

export function parse_ident__lto_1ba4622a(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["err"]("expected identifier, got EOF", state_pos(st2)));
    } else {
      return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_46) => {
        if (__cps_v_46) {
          const start = state_pos(st2);
          return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (end_st) => {
            return __k(ParseResult["ok"](String.slice(state_src(st2), start, state_pos(end_st)), end_st));
          });
        } else {
          return __k(ParseResult["err"](((__lto_self_2988) => {
            return (__lto_self_2988 + "'");
          })(((__lto_self_2990) => {
            return (__lto_self_2990 + state_peek__lto_9309ae26(st2));
          })("expected identifier, got '")), state_pos(st2)));
        }
      });
    }
  });
}

export function expect__lto_f3280589(st, expected) {
  const st2 = skip_ws__lto_1bb67705(st);
  const len = String.len(expected);
  const src = state_src(st2);
  const pos = state_pos(st2);
  const __lto_a_3002 = (String.len(src) - pos);
  const __match_413 = ((__lto_a_3002 < len) ? Ordering["less"] : ((__match_412) => {
    if (__match_412) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_3002, len)));
  if (((__match_413[LUMO_TAG] === "less") ? false : ((__match_413[LUMO_TAG] === "equal") ? true : true))) {
    const slice = String.slice(src, pos, ((__lto_self_3004) => {
      return (__lto_self_3004 + len);
    })(pos));
    if ((slice === expected)) {
      return ParseResult["ok"](expected, state_advance__lto_92991de6(st2, len));
    } else {
      return ParseResult["err"](((__lto_self_3012) => {
        return (__lto_self_3012 + "'");
      })(((__lto_self_3014) => {
        return (__lto_self_3014 + slice);
      })(((__lto_self_3016) => {
        return (__lto_self_3016 + "', got '");
      })(((__lto_self_3018) => {
        return (__lto_self_3018 + expected);
      })("expected '")))), pos);
    }
  } else {
    return ParseResult["err"](((__lto_self_3028) => {
      return (__lto_self_3028 + "'");
    })(((__lto_self_3030) => {
      return (__lto_self_3030 + expected);
    })("expected '")), pos);
  }
}

export function parse_quoted__lto_38e07bea(st) {
  const st2 = skip_ws__lto_1bb67705(st);
  if ((state_peek__lto_9309ae26(st2) === "'")) {
    const end_st = scan_until_quote__lto_3890158f(state_advance__lto_92991de6(st2, 1));
    return ParseResult["ok"](String.slice(state_src(st2), (state_pos(st2) + 1), state_pos(end_st)), state_advance__lto_92991de6(end_st, 1));
  } else {
    return ParseResult["err"]("expected quoted literal", state_pos(st2));
  }
}

export function scan_until_quote__lto_3890158f(st) {
  if (state_eof__lto_9309ae26(st)) {
    return st;
  } else if ((state_peek__lto_9309ae26(st) === "'")) {
    return st;
  } else {
    return scan_until_quote__lto_3890158f(state_advance__lto_92991de6(st, 1));
  }
}

export function peek_is_word__lto_1bb67705(st, word) {
  const st2 = skip_ws__lto_1bb67705(st);
  const src = state_src(st2);
  const pos = state_pos(st2);
  const len = String.len(word);
  const __lto_a_3054 = (pos + len);
  const __lto_b_3055 = String.len(src);
  const __match_421 = ((__lto_a_3054 < __lto_b_3055) ? Ordering["less"] : ((__match_420) => {
    if (__match_420) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_3054, __lto_b_3055)));
  if (((__match_421[LUMO_TAG] === "less") ? false : ((__match_421[LUMO_TAG] === "equal") ? false : true))) {
    return false;
  } else if ((String.slice(src, pos, ((__lto_self_3056) => {
    return (__lto_self_3056 + len);
  })(pos)) === word)) {
    let __match_427;
    let __match_426;
    const __lto_a_3070 = (pos + len);
    const __lto_b_3071 = String.len(src);
    if ((__lto_a_3070 < __lto_b_3071)) {
      __match_426 = Ordering["less"];
    } else if ((__lto_a_3070 === __lto_b_3071)) {
      __match_426 = Ordering["equal"];
    } else {
      __match_426 = Ordering["greater"];
    }
    if ((__match_426[LUMO_TAG] === "less")) {
      __match_427 = false;
    } else if ((__match_426[LUMO_TAG] === "equal")) {
      __match_427 = true;
    } else {
      __match_427 = true;
    }
    if (__match_427) {
      return true;
    } else if (is_ident_continue__lto_3890158f(String.char_at(src, ((__lto_self_3072) => {
      return (__lto_self_3072 + len);
    })(pos)))) {
      return false;
    } else {
      return true;
    }
  } else {
    return false;
  }
}

export function peek_is_pratt_item_start__lto_94a384aa(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const code = String.char_code_at(state_peek__lto_9309ae26(st2), 0);
    const __k_148 = (is_upper) => {
      if (is_upper) {
        return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (st3) => {
          return __k(((a, b) => {
            return (a === b);
          })(state_peek__lto_9309ae26(skip_ws__lto_1bb67705(st3)), ":"));
        });
      } else {
        return __k(false);
      }
    };
    const __match_436 = ((code < 65) ? Ordering["less"] : ((__match_435) => {
      if (__match_435) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if (((__match_436[LUMO_TAG] === "less") ? false : ((__match_436[LUMO_TAG] === "equal") ? true : true))) {
      const __match_431 = ((code < 90) ? Ordering["less"] : ((__match_433) => {
        if (__match_433) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(code, 90)));
      if ((__match_431[LUMO_TAG] === "less")) {
        return __k_148(true);
      } else if ((__match_431[LUMO_TAG] === "equal")) {
        return __k_148(true);
      } else {
        return __k_148(false);
      }
    } else {
      return __k_148(false);
    }
  });
}

export function peek_is_bp_marker__lto_3890158f(st) {
  const st2 = skip_ws__lto_1bb67705(st);
  if (peek_is_word__lto_1bb67705(st2, "bp")) {
    return (state_peek__lto_9309ae26(skip_ws__lto_1bb67705(state_advance__lto_92991de6(st2, 2))) === "(");
  } else {
    return false;
  }
}

export function peek_is_rule_start__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_47) => {
      if (__cps_v_47) {
        return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (st3) => {
          return __k(((a, b) => {
            return (a === b);
          })(state_peek__lto_9309ae26(skip_ws__lto_1bb67705(st3)), "="));
        });
      } else {
        return __k(false);
      }
    });
  });
}

export function has_alpha__lto_090deca7(s, i) {
  const __lto_b_3099 = String.len(s);
  const __match_441 = ((i < __lto_b_3099) ? Ordering["less"] : ((__match_440) => {
    if (__match_440) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3099)));
  if (((__match_441[LUMO_TAG] === "less") ? false : ((__match_441[LUMO_TAG] === "equal") ? true : true))) {
    return false;
  } else if (is_alpha__lto_9309ae26(String.char_at(s, i))) {
    return true;
  } else {
    return has_alpha__lto_090deca7(s, ((__lto_self_3100) => {
      return (__lto_self_3100 + 1);
    })(i));
  }
}

export function parse_grammar_items__lto_3890158f(__caps, st, tokens, attrs, rules, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_attr(attrs), list_reverse_rule(rules)), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "@")) {
      return parse_token_def(__caps, st2, (__cps_v_50) => {
        if ((__cps_v_50[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_50.args[1], list_concat_string(__cps_v_50.args[0], tokens), attrs, rules, __k);
        } else {
          return __k(ParseResult["err"](__cps_v_50.args[0], __cps_v_50.args[1]));
        }
      });
    } else if ((state_peek__lto_9309ae26(st2) === "#")) {
      return parse_grammar_attr__lto_8227044e(__caps, st2, (__cps_v_49) => {
        if ((__cps_v_49[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_49.args[1], tokens, List["cons"](__cps_v_49.args[0], attrs), rules, __k);
        } else {
          return __k(ParseResult["err"](__cps_v_49.args[0], __cps_v_49.args[1]));
        }
      });
    } else {
      return parse_rule(__caps, st2, (__cps_v_48) => {
        if ((__cps_v_48[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_48.args[1], tokens, attrs, List["cons"](__cps_v_48.args[0], rules), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_48.args[0], __cps_v_48.args[1]));
        }
      });
    }
  });
}

export function parse_grammar_attr__lto_8227044e(__caps, st, __k) {
  return __thunk(() => {
    const __match_450 = expect__lto_f3280589(st, "#");
    if ((__match_450[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_450.args[0], __match_450.args[1]));
    } else {
      const __match_451 = expect__lto_f3280589(__match_450.args[1], "[");
      if ((__match_451[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_451.args[0], __match_451.args[1]));
      } else {
        return parse_ident__lto_1ba4622a(__caps, __match_451.args[1], (__cps_v_52) => {
          if ((__cps_v_52[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_52.args[0], __cps_v_52.args[1]));
          } else {
            const attr_name = __cps_v_52.args[0];
            const st4 = __cps_v_52.args[1];
            if ((attr_name === "parser")) {
              const __match_454 = expect__lto_f3280589(st4, "(");
              if ((__match_454[LUMO_TAG] === "err")) {
                return __k(ParseResult["err"](__match_454.args[0], __match_454.args[1]));
              } else {
                return parse_parser_attr_args__lto_3890158f(__caps, __match_454.args[1], false, "", (__cps_v_51) => {
                  if ((__cps_v_51[LUMO_TAG] === "err")) {
                    return __k(ParseResult["err"](__cps_v_51.args[0], __cps_v_51.args[1]));
                  } else {
                    const __match_456 = expect__lto_f3280589(__cps_v_51.args[1], ")");
                    if ((__match_456[LUMO_TAG] === "err")) {
                      return __k(ParseResult["err"](__match_456.args[0], __match_456.args[1]));
                    } else {
                      const __match_457 = expect__lto_f3280589(__match_456.args[1], "]");
                      if ((__match_457[LUMO_TAG] === "err")) {
                        return __k(ParseResult["err"](__match_457.args[0], __match_457.args[1]));
                      } else {
                        return __k(ParseResult["ok"](GrammarAttr["parser_generate"](__cps_v_51.args[0]), __match_457.args[1]));
                      }
                    }
                  }
                });
              }
            } else {
              return __k(ParseResult["err"](((__lto_self_3116) => {
                return (__lto_self_3116 + attr_name);
              })("unknown file attribute: "), state_pos(st4)));
            }
          }
        });
      }
    }
  });
}

export function parse_parser_attr_args__lto_3890158f(__caps, st, has_path, path, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === ")")) {
      return __k(ParseResult["ok"](path, st2));
    } else if ((state_peek__lto_9309ae26(st2) === ",")) {
      return parse_parser_attr_args__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), has_path, path, __k);
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_54) => {
        if ((__cps_v_54[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_54.args[0], __cps_v_54.args[1]));
        } else {
          const __match_461 = expect__lto_f3280589(__cps_v_54.args[1], "=");
          if ((__match_461[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__match_461.args[0], __match_461.args[1]));
          } else {
            const st4 = __match_461.args[1];
            if ((__cps_v_54.args[0] === "path")) {
              const __match_464 = parse_string_lit__lto_38e07bea(st4);
              if ((__match_464[LUMO_TAG] === "err")) {
                return __k(ParseResult["err"](__match_464.args[0], __match_464.args[1]));
              } else {
                return parse_parser_attr_args__lto_3890158f(__caps, __match_464.args[1], true, __match_464.args[0], __k);
              }
            } else {
              return try_skip_attr_value__lto_3890158f(__caps, st4, (__cps_v_53) => {
                if ((__cps_v_53[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_53.args[0], __cps_v_53.args[1]));
                } else {
                  return parse_parser_attr_args__lto_3890158f(__caps, __cps_v_53.args[1], has_path, path, __k);
                }
              });
            }
          }
        }
      });
    }
  });
}

export function try_skip_attr_value__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === "\"")) {
      const __match_467 = parse_string_lit__lto_38e07bea(st2);
      if ((__match_467[LUMO_TAG] === "ok")) {
        return __k(ParseResult["ok"]("", __match_467.args[1]));
      } else {
        return __k(ParseResult["err"](__match_467.args[0], __match_467.args[1]));
      }
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_55) => {
        if ((__cps_v_55[LUMO_TAG] === "ok")) {
          return __k(ParseResult["ok"]("", __cps_v_55.args[1]));
        } else {
          return __k(ParseResult["err"](__cps_v_55.args[0], __cps_v_55.args[1]));
        }
      });
    }
  });
}

export function parse_token_names__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
    } else {
      return peek_is_rule_start__lto_3890158f(__caps, st2, (__cps_v_58) => {
        if (__cps_v_58) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((state_peek__lto_9309ae26(st2) === "@")) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else {
          return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_57) => {
            if (__cps_v_57) {
              return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_56) => {
                if ((__cps_v_56[LUMO_TAG] === "ok")) {
                  return parse_token_names__lto_3890158f(__caps, __cps_v_56.args[1], List["cons"](__cps_v_56.args[0], acc), __k);
                } else {
                  return __k(ParseResult["err"](__cps_v_56.args[0], __cps_v_56.args[1]));
                }
              });
            } else {
              return __k(ParseResult["ok"](list_reverse_string(acc), st2));
            }
          });
        }
      });
    }
  });
}

export function parse_rule_body__lto_3890158f(__caps, st, rule_name, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_is_pratt(__caps, st2, (__cps_v_60) => {
      if (__cps_v_60) {
        return parse_pratt_body(__caps, st2, __k);
      } else {
        return peek_char(__caps, st2, (__lto_self_3140) => {
          if ((__lto_self_3140 === "|")) {
            return parse_alternatives(__caps, st2, __k);
          } else {
            return parse_sequence(__caps, st2, __k);
          }
        });
      }
    });
  });
}

export function parse_pratt_items__lto_3890158f(__caps, st, atoms, alts, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["err"]("unexpected EOF in pratt block", state_pos(st2)));
    } else if ((state_peek__lto_9309ae26(st2) === "}")) {
      return __k(ParseResult["ok"](RuleBody["pratt"](list_reverse_string(atoms), list_reverse_pratt_alt(alts)), st2));
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_63) => {
        if ((__cps_v_63[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_63.args[0], __cps_v_63.args[1]));
        } else {
          const name = __cps_v_63.args[0];
          const __match_478 = expect__lto_f3280589(__cps_v_63.args[1], ":");
          if ((__match_478[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__match_478.args[0], __match_478.args[1]));
          } else {
            const st4 = __match_478.args[1];
            if ((name === "atom")) {
              return parse_pratt_atom_list__lto_3890158f(__caps, st4, List["nil"], (__cps_v_62) => {
                if ((__cps_v_62[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_62.args[0], __cps_v_62.args[1]));
                } else {
                  return parse_pratt_items__lto_3890158f(__caps, __cps_v_62.args[1], list_concat_string(list_reverse_string(__cps_v_62.args[0]), atoms), alts, __k);
                }
              });
            } else {
              return parse_pratt_alt_body(__caps, st4, name, (__cps_v_61) => {
                if ((__cps_v_61[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_61.args[0], __cps_v_61.args[1]));
                } else {
                  return parse_pratt_items__lto_3890158f(__caps, __cps_v_61.args[1], atoms, List["cons"](__cps_v_61.args[0], alts), __k);
                }
              });
            }
          }
        }
      });
    }
  });
}

export function parse_pratt_atom_list__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "}")) {
      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
    } else {
      return peek_is_pratt_item_start__lto_94a384aa(__caps, st2, (__cps_v_67) => {
        if (__cps_v_67) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((state_peek__lto_9309ae26(st2) === "|")) {
          return parse_ident__lto_1ba4622a(__caps, state_advance__lto_92991de6(st2, 1), (__cps_v_66) => {
            if ((__cps_v_66[LUMO_TAG] === "ok")) {
              return parse_pratt_atom_list__lto_3890158f(__caps, __cps_v_66.args[1], List["cons"](__cps_v_66.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_66.args[0], __cps_v_66.args[1]));
            }
          });
        } else {
          return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_65) => {
            if (__cps_v_65) {
              return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_64) => {
                if ((__cps_v_64[LUMO_TAG] === "ok")) {
                  return parse_pratt_atom_list__lto_3890158f(__caps, __cps_v_64.args[1], List["cons"](__cps_v_64.args[0], acc), __k);
                } else {
                  return __k(ParseResult["err"](__cps_v_64.args[0], __cps_v_64.args[1]));
                }
              });
            } else {
              return __k(ParseResult["ok"](list_reverse_string(acc), st2));
            }
          });
        }
      });
    }
  });
}

export function parse_pratt_pattern__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "}")) {
      return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
    } else {
      return peek_is_pratt_item_start__lto_94a384aa(__caps, st2, (__cps_v_69) => {
        if (__cps_v_69) {
          return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
        } else if (peek_is_bp_marker__lto_3890158f(st2)) {
          return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
        } else {
          return parse_pratt_pattern_element__lto_3890158f(__caps, st2, (__cps_v_68) => {
            if ((__cps_v_68[LUMO_TAG] === "ok")) {
              return parse_pratt_pattern__lto_3890158f(__caps, __cps_v_68.args[1], List["cons"](__cps_v_68.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_68.args[0], __cps_v_68.args[1]));
            }
          });
        }
      });
    }
  });
}

export function parse_pratt_pattern_element__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === "'")) {
      const __match_501 = parse_quoted__lto_38e07bea(st2);
      if ((__match_501[LUMO_TAG] === "ok")) {
        return classify_literal(__caps, __match_501.args[0], (__cps_v_74) => {
          return __k(apply_postfix_elem__lto_3890158f(Element["token"](__cps_v_74), __match_501.args[1]));
        });
      } else {
        return __k(ParseResult["err"](__match_501.args[0], __match_501.args[1]));
      }
    } else if ((state_peek__lto_9309ae26(st2) === "(")) {
      return parse_pratt_group__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), List["nil"], (__cps_v_72) => {
        if ((__cps_v_72[LUMO_TAG] === "ok")) {
          const __match_500 = expect__lto_f3280589(__cps_v_72.args[1], ")");
          if ((__match_500[LUMO_TAG] === "ok")) {
            return __k(apply_postfix_elem__lto_3890158f(Element["group"](__cps_v_72.args[0]), __match_500.args[1]));
          } else {
            return __k(ParseResult["err"](__match_500.args[0], __match_500.args[1]));
          }
        } else {
          return __k(ParseResult["err"](__cps_v_72.args[0], __cps_v_72.args[1]));
        }
      });
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_71) => {
        if ((__cps_v_71[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_71.args[0], __cps_v_71.args[1]));
        } else {
          const name = __cps_v_71.args[0];
          const st3 = __cps_v_71.args[1];
          if ((state_peek__lto_9309ae26(st3) === ":")) {
            return parse_pratt_pattern_element__lto_3890158f(__caps, state_advance__lto_92991de6(st3, 1), (__cps_v_70) => {
              if ((__cps_v_70[LUMO_TAG] === "ok")) {
                return __k(apply_postfix_elem__lto_3890158f(Element["labeled"](name, __cps_v_70.args[0]), __cps_v_70.args[1]));
              } else {
                return __k(ParseResult["err"](__cps_v_70.args[0], __cps_v_70.args[1]));
              }
            });
          } else {
            return __k(apply_postfix_elem__lto_3890158f(Element["node"](NodeRef["mk"](name)), st3));
          }
        }
      });
    }
  });
}

export function parse_pratt_group__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === ")")) {
      return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "|")) {
      return parse_pratt_group__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), acc, __k);
    } else {
      return parse_pratt_pattern_element__lto_3890158f(__caps, st2, (__cps_v_75) => {
        if ((__cps_v_75[LUMO_TAG] === "ok")) {
          return parse_pratt_group__lto_3890158f(__caps, __cps_v_75.args[1], List["cons"](__cps_v_75.args[0], acc), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_75.args[0], __cps_v_75.args[1]));
        }
      });
    }
  });
}

export function parse_number__lto_1ba4622a(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["err"]("expected number, got EOF", state_pos(st2)));
    } else if (is_digit__lto_9309ae26(state_peek__lto_9309ae26(st2))) {
      const start = state_pos(st2);
      return scan_digits(__caps, state_advance__lto_92991de6(st2, 1), (end_st) => {
        return __k(ParseResult["ok"](parse_int__lto_1856fa45(String.slice(state_src(st2), start, state_pos(end_st)), 0, 0), end_st));
      });
    } else {
      return __k(ParseResult["err"](((__lto_self_3184) => {
        return (__lto_self_3184 + "'");
      })(((__lto_self_3186) => {
        return (__lto_self_3186 + state_peek__lto_9309ae26(st2));
      })("expected number, got '")), state_pos(st2)));
    }
  });
}

export function parse_int__lto_1856fa45(s, i, acc) {
  const __lto_b_3195 = String.len(s);
  const __match_509 = ((i < __lto_b_3195) ? Ordering["less"] : ((__match_508) => {
    if (__match_508) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3195)));
  if (((__match_509[LUMO_TAG] === "less") ? false : ((__match_509[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const digit = (String.char_code_at(s, i) - 48);
    return parse_int__lto_1856fa45(s, ((__lto_self_3200) => {
      return (__lto_self_3200 + 1);
    })(i), ((__lto_self_3204) => {
      return (__lto_self_3204 + digit);
    })(((__lto_self_3206) => {
      return (__lto_self_3206 * 10);
    })(acc)));
  }
}

export function parse_alt_items__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_char(__caps, st2, (__lto_self_3212) => {
      if ((__lto_self_3212 === "|")) {
        const st3 = state_advance__lto_92991de6(skip_ws__lto_1bb67705(st2), 1);
        const st4 = skip_ws__lto_1bb67705(st3);
        if ((state_peek__lto_9309ae26(st4) === "'")) {
          const __match_514 = parse_quoted__lto_38e07bea(st4);
          if ((__match_514[LUMO_TAG] === "ok")) {
            return parse_alt_items__lto_3890158f(__caps, __match_514.args[1], List["cons"](Alternative["mk"](__match_514.args[0]), acc), __k);
          } else {
            return __k(ParseResult["err"](__match_514.args[0], __match_514.args[1]));
          }
        } else {
          return parse_ident__lto_1ba4622a(__caps, st3, (__cps_v_76) => {
            if ((__cps_v_76[LUMO_TAG] === "ok")) {
              return parse_alt_items__lto_3890158f(__caps, __cps_v_76.args[1], List["cons"](Alternative["mk"](__cps_v_76.args[0]), acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_76.args[0], __cps_v_76.args[1]));
            }
          });
        }
      } else {
        return __k(ParseResult["ok"](RuleBody["alternatives"](list_reverse_alt(acc)), st2));
      }
    });
  });
}

export function is_seq_terminator__lto_3890158f(__caps, st, __k) {
  return peek_char(__caps, st, (c) => {
    if ((c === ")")) {
      return __k(true);
    } else {
      return peek_is_rule_start__lto_3890158f(__caps, st, (__cps_v_78) => {
        if (__cps_v_78) {
          return __k(true);
        } else if ((c === "@")) {
          return __k(true);
        } else {
          return __k(false);
        }
      });
    }
  });
}

export function apply_postfix_elem__lto_3890158f(elem, st) {
  if (state_eof__lto_9309ae26(st)) {
    return ParseResult["ok"](elem, st);
  } else if ((state_peek__lto_9309ae26(st) === "?")) {
    return apply_postfix_elem__lto_3890158f(Element["optional"](elem), state_advance__lto_92991de6(st, 1));
  } else if ((state_peek__lto_9309ae26(st) === "*")) {
    return apply_postfix_elem__lto_3890158f(Element["repeated"](elem), state_advance__lto_92991de6(st, 1));
  } else {
    return ParseResult["ok"](elem, st);
  }
}

export function parse_atom__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === "'")) {
      const __match_528 = parse_quoted__lto_38e07bea(st2);
      if ((__match_528[LUMO_TAG] === "ok")) {
        return classify_literal(__caps, __match_528.args[0], (__cps_v_83) => {
          return __k(ParseResult["ok"](Element["token"](__cps_v_83), __match_528.args[1]));
        });
      } else {
        return __k(ParseResult["err"](__match_528.args[0], __match_528.args[1]));
      }
    } else if ((state_peek__lto_9309ae26(st2) === "(")) {
      return parse_group_elements__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), List["nil"], (__cps_v_81) => {
        if ((__cps_v_81[LUMO_TAG] === "ok")) {
          const __match_527 = expect__lto_f3280589(__cps_v_81.args[1], ")");
          if ((__match_527[LUMO_TAG] === "ok")) {
            return __k(ParseResult["ok"](Element["group"](__cps_v_81.args[0]), __match_527.args[1]));
          } else {
            return __k(ParseResult["err"](__match_527.args[0], __match_527.args[1]));
          }
        } else {
          return __k(ParseResult["err"](__cps_v_81.args[0], __cps_v_81.args[1]));
        }
      });
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_80) => {
        if ((__cps_v_80[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_80.args[0], __cps_v_80.args[1]));
        } else {
          const name = __cps_v_80.args[0];
          const st3 = __cps_v_80.args[1];
          if ((state_peek__lto_9309ae26(st3) === ":")) {
            return parse_element(__caps, state_advance__lto_92991de6(st3, 1), (__cps_v_79) => {
              if ((__cps_v_79[LUMO_TAG] === "ok")) {
                return __k(ParseResult["ok"](Element["labeled"](name, __cps_v_79.args[0]), __cps_v_79.args[1]));
              } else {
                return __k(ParseResult["err"](__cps_v_79.args[0], __cps_v_79.args[1]));
              }
            });
          } else {
            return __k(ParseResult["ok"](Element["node"](NodeRef["mk"](name)), st3));
          }
        }
      });
    }
  });
}

export function parse_group_elements__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if ((state_peek__lto_9309ae26(st2) === ")")) {
      return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
    } else {
      return parse_element(__caps, st2, (__cps_v_84) => {
        if ((__cps_v_84[LUMO_TAG] === "ok")) {
          return parse_group_elements__lto_3890158f(__caps, __cps_v_84.args[1], List["cons"](__cps_v_84.args[0], acc), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_84.args[0], __cps_v_84.args[1]));
        }
      });
    }
  });
}

export function list_contains_string__lto_3890158f(xs, target) {
  if ((xs[LUMO_TAG] === "nil")) {
    return false;
  } else if ((xs.args[0] === target)) {
    return true;
  } else {
    return list_contains_string__lto_3890158f(xs.args[1], target);
  }
}

export function parse_string_lit__lto_38e07bea(st) {
  const st2 = skip_ws__lto_1bb67705(st);
  if ((state_peek__lto_9309ae26(st2) === "\"")) {
    const end_st = scan_until_double_quote__lto_3890158f(state_advance__lto_92991de6(st2, 1));
    return ParseResult["ok"](String.slice(state_src(st2), (state_pos(st2) + 1), state_pos(end_st)), state_advance__lto_92991de6(end_st, 1));
  } else {
    return ParseResult["err"]("expected string literal", state_pos(st2));
  }
}

export function scan_until_double_quote__lto_3890158f(st) {
  if (state_eof__lto_9309ae26(st)) {
    return st;
  } else if ((state_peek__lto_9309ae26(st) === "\"")) {
    return st;
  } else {
    return scan_until_double_quote__lto_3890158f(state_advance__lto_92991de6(st, 1));
  }
}

main();
