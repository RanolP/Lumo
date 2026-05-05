const LUMO_TAG = Symbol.for("Lumo/tag");
const __lumo_error = () => { throw new Error("lumo runtime error"); };
const __thunk = (fn) => { fn.__t = 1; return fn; };
const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };
const __identity = (__v) => __v;

import { readFileSync as __lumo_readFileSync, writeFileSync as __lumo_writeFileSync } from "node:fs";




export const CollectedTokens = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const StringPair = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };

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

export function has_labeled_element(elem) {
  if ((elem[LUMO_TAG] === "labeled")) {
    return true;
  } else if ((elem[LUMO_TAG] === "group")) {
    return has_labeled_elements(elem.args[0]);
  } else {
    return ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
      return has_labeled_element(inner);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
      return has_labeled_element(inner);
    })(elem.args[0]) : false));
  }
}

export function has_labeled_elements(elems) {
  if ((elems[LUMO_TAG] === "nil")) {
    return false;
  } else if (has_labeled_element(elems.args[0])) {
    return true;
  } else {
    return has_labeled_elements(elems.args[1]);
  }
}

export function emit_accessors_for_elements(__caps, s, elems, token_defs, __k) {
  return emit_accessors_for_elements_ctx(__caps, s, elems, token_defs, "", __k);
}

export function emit_accessors_for_elements_ctx(__caps, s, elems, token_defs, prev_kw, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = elems.args[1];
      const __match_33 = elems.args[0];
      if ((__match_33[LUMO_TAG] === "labeled")) {
        return emit_single_accessor_ctx__lto_3890158f(__caps, s, __match_33.args[0], __match_33.args[1], token_defs, prev_kw, (s2) => {
          return emit_accessors_for_elements_ctx(__caps, s2, rest, token_defs, "", __k);
        });
      } else if ((__match_33[LUMO_TAG] === "token")) {
        const __k_24 = (new_kw) => {
          return emit_accessors_for_elements_ctx(__caps, s, rest, token_defs, new_kw, __k);
        };
        const __match_34 = __match_33.args[0];
        if ((__match_34[LUMO_TAG] === "keyword")) {
          return __k_24(__match_34.args[0]);
        } else {
          return __k_24("");
        }
      } else {
        return ((__match_33[LUMO_TAG] === "optional") ? ((inner) => {
          return emit_accessors_for_element_ctx(__caps, s, inner, token_defs, prev_kw, (s2) => {
            return emit_accessors_for_elements_ctx(__caps, s2, rest, token_defs, "", __k);
          });
        })(__match_33.args[0]) : ((__match_33[LUMO_TAG] === "group") ? ((inner_elems) => {
          return emit_accessors_for_elements_ctx(__caps, s, inner_elems, token_defs, prev_kw, (s2) => {
            return emit_accessors_for_elements_ctx(__caps, s2, rest, token_defs, "", __k);
          });
        })(__match_33.args[0]) : ((__match_33[LUMO_TAG] === "repeated") ? ((inner) => {
          return emit_accessors_for_element_repeated(__caps, s, inner, token_defs, (s2) => {
            return emit_accessors_for_elements_ctx(__caps, s2, rest, token_defs, "", __k);
          });
        })(__match_33.args[0]) : emit_accessors_for_elements_ctx(__caps, s, rest, token_defs, "", __k))));
      }
    }
  });
}

export function emit_accessors_for_element_ctx(__caps, s, elem, token_defs, prev_kw, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "labeled")) {
      return emit_single_accessor_ctx__lto_3890158f(__caps, s, elem.args[0], elem.args[1], token_defs, prev_kw, __k);
    } else if ((elem[LUMO_TAG] === "group")) {
      return emit_accessors_for_elements_ctx(__caps, s, elem.args[0], token_defs, prev_kw, __k);
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_accessors_for_element_ctx(__caps, s, inner, token_defs, prev_kw, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_accessors_for_element_repeated(__caps, s, inner, token_defs, __k);
      })(elem.args[0]) : __k(s)));
    }
  });
}

export function emit_accessors_for_element_repeated(__caps, s, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "labeled")) {
      return emit_single_accessor_repeated(__caps, s, elem.args[0], elem.args[1], token_defs, __k);
    } else if ((elem[LUMO_TAG] === "group")) {
      return emit_accessors_for_elements_repeated(__caps, s, elem.args[0], token_defs, __k);
    } else {
      return ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_accessors_for_element_repeated(__caps, s, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_accessors_for_element_repeated(__caps, s, inner, token_defs, __k);
      })(elem.args[0]) : __k(s)));
    }
  });
}

export function emit_accessors_for_elements_repeated(__caps, s, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return emit_accessors_for_element_repeated(__caps, s, elems.args[0], token_defs, (s2) => {
        return emit_accessors_for_elements_repeated(__caps, s2, elems.args[1], token_defs, __k);
      });
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
        return emit_accessors_for_elements(__caps, s, elems, token_defs, __k);
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
        return emit_accessors_for_elements_repeated(__caps, s, elems, token_defs, __k);
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

export function keyword_to_syntax_kind_name(__caps, kw, __k) {
  return keyword_variant__lto_1ba4622a(__caps, kw, __k);
}

export function emit_parse_rules(__caps, s, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __match_46 = rules.args[0];
      const name = __match_46.args[0];
      const body = __match_46.args[1];
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
      return make_first_elem_lookahead__lto_8227044e(__caps, body.args[0], token_defs, __k);
    } else if ((body[LUMO_TAG] === "alternatives")) {
      return make_alts_lookahead__lto_1ba4622a(__caps, body.args[0], __k);
    } else {
      return make_pratt_lookahead__lto_8227044e(__caps, body.args[0], body.args[1], __k);
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

export function elems_have_expr(__caps, elems, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(false);
    } else {
      return elem_has_expr__lto_3890158f(__caps, elems.args[0], (__cps_v_9) => {
        if (__cps_v_9) {
          return __k(true);
        } else {
          return elems_have_expr(__caps, elems.args[1], __k);
        }
      });
    }
  });
}

export function emit_parse_elements(__caps, s, elems, token_defs, indent, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return emit_parse_element__lto_8227044e(__caps, s, elems.args[0], token_defs, indent, (s2) => {
        return emit_parse_elements(__caps, s2, elems.args[1], token_defs, indent, __k);
      });
    }
  });
}

export function make_group_lookahead(__caps, elems, token_defs, __k) {
  return make_first_elem_lookahead__lto_8227044e(__caps, elems, token_defs, __k);
}

export function emit_pratt_alt_kinds(__caps, s, alts, __k) {
  return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, s, alts, List["nil"], __k);
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

export function to_snake(__caps, name, __k) {
  return __thunk(() => {
    return __k(to_snake_loop__lto_1fab3ad0(name, 0, ""));
  });
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
      const __match_61 = expect__lto_f3280589(state_advance__lto_92991de6(st2, 2), "(");
      if ((__match_61[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_61.args[0], __match_61.args[1]));
      } else {
        return parse_bp_val(__caps, __match_61.args[1], (__cps_v_10) => {
          if ((__cps_v_10[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_10.args[0], __cps_v_10.args[1]));
          } else {
            const __match_63 = expect__lto_f3280589(__cps_v_10.args[1], ")");
            if ((__match_63[LUMO_TAG] === "err")) {
              return __k(ParseResult["err"](__match_63.args[0], __match_63.args[1]));
            } else {
              return __k(ParseResult["ok"](__cps_v_10.args[0], __match_63.args[1]));
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
    const __match_65 = expect__lto_f3280589(st, "@token");
    if ((__match_65[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_65.args[0], __match_65.args[1]));
    } else {
      return parse_token_names__lto_3890158f(__caps, __match_65.args[1], List["nil"], __k);
    }
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident__lto_1ba4622a(__caps, st, (__cps_v_12) => {
    if ((__cps_v_12[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_12.args[0], __cps_v_12.args[1]));
    } else {
      const name = __cps_v_12.args[0];
      const __match_67 = expect__lto_f3280589(__cps_v_12.args[1], "=");
      if ((__match_67[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_67.args[0], __match_67.args[1]));
      } else {
        return parse_rule_body__lto_3890158f(__caps, __match_67.args[1], name, (__cps_v_11) => {
          if ((__cps_v_11[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_11.args[0], __cps_v_11.args[1]));
          } else {
            return __k(ParseResult["ok"](Rule["mk"](name, __cps_v_11.args[0]), __cps_v_11.args[1]));
          }
        });
      }
    }
  });
}

export function parse_pratt_body(__caps, st, __k) {
  return __thunk(() => {
    const __match_69 = expect__lto_f3280589(st, "pratt");
    if ((__match_69[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_69.args[0], __match_69.args[1]));
    } else {
      const __match_70 = expect__lto_f3280589(__match_69.args[1], "{");
      if ((__match_70[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_70.args[0], __match_70.args[1]));
      } else {
        return parse_pratt_items__lto_3890158f(__caps, __match_70.args[1], List["nil"], List["nil"], (__cps_v_13) => {
          if ((__cps_v_13[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_13.args[0], __cps_v_13.args[1]));
          } else {
            const __match_72 = expect__lto_f3280589(__cps_v_13.args[1], "}");
            if ((__match_72[LUMO_TAG] === "err")) {
              return __k(ParseResult["err"](__match_72.args[0], __match_72.args[1]));
            } else {
              return __k(ParseResult["ok"](__cps_v_13.args[0], __match_72.args[1]));
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
      return parse_pratt_pattern__lto_3890158f(__caps, lbp_res.args[1], List["nil"], (__cps_v_17) => {
        if ((__cps_v_17[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_17.args[0], __cps_v_17.args[1]));
        } else {
          const elems = __cps_v_17.args[0];
          const st3 = __cps_v_17.args[1];
          return try_parse_bp(__caps, st3, (__cps_v_16) => {
            if ((__cps_v_16[LUMO_TAG] === "ok")) {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](lbp, __cps_v_16.args[0])), __cps_v_16.args[1]));
            } else {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](lbp, BpVal["none"])), st3));
            }
          });
        }
      });
    } else {
      return parse_pratt_pattern__lto_3890158f(__caps, st, List["nil"], (__cps_v_15) => {
        if ((__cps_v_15[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_15.args[0], __cps_v_15.args[1]));
        } else {
          const elems = __cps_v_15.args[0];
          const st2 = __cps_v_15.args[1];
          return try_parse_bp(__caps, st2, (__cps_v_14) => {
            if ((__cps_v_14[LUMO_TAG] === "ok")) {
              return __k(ParseResult["ok"](PrattAlt["mk"](name, elems, BindingPower["mk"](BpVal["none"], __cps_v_14.args[0])), __cps_v_14.args[1]));
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
      return parse_number__lto_1ba4622a(__caps, st2, (__cps_v_18) => {
        if ((__cps_v_18[LUMO_TAG] === "ok")) {
          return __k(ParseResult["ok"](BpVal["num"](__cps_v_18.args[0]), __cps_v_18.args[1]));
        } else {
          return __k(ParseResult["err"](__cps_v_18.args[0], __cps_v_18.args[1]));
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
      return is_seq_terminator__lto_3890158f(__caps, st2, (__cps_v_20) => {
        if (__cps_v_20) {
          return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
        } else {
          return parse_element(__caps, st2, (__cps_v_19) => {
            if ((__cps_v_19[LUMO_TAG] === "ok")) {
              return parse_seq_elements(__caps, __cps_v_19.args[1], List["cons"](__cps_v_19.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_19.args[0], __cps_v_19.args[1]));
            }
          });
        }
      });
    }
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom__lto_3890158f(__caps, st, (__cps_v_21) => {
    if ((__cps_v_21[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_21.args[0], __cps_v_21.args[1]));
    } else {
      return __k(apply_postfix_elem__lto_3890158f(__cps_v_21.args[0], __cps_v_21.args[1]));
    }
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    const token_defs = g.args[0];
    return resolve_rules(__caps, token_defs, g.args[2], (__cps_v_22) => {
      return __k(Grammar["mk"](token_defs, g.args[1], __cps_v_22));
    });
  });
}

export function resolve_rules(__caps, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const __match_88 = rules.args[0];
      return resolve_body(__caps, token_defs, __match_88.args[1], (resolved_body) => {
        return resolve_rules(__caps, token_defs, rules.args[1], (__cps_v_23) => {
          return __k(List["cons"](Rule["mk"](__match_88.args[0], resolved_body), __cps_v_23));
        });
      });
    }
  });
}

export function resolve_body(__caps, token_defs, body, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return resolve_elements(__caps, token_defs, body.args[0], (__cps_v_25) => {
        return __k(RuleBody["sequence"](__cps_v_25));
      });
    } else if ((body[LUMO_TAG] === "alternatives")) {
      const alts = body.args[0];
      return __k(body);
    } else {
      return resolve_pratt_alts(__caps, token_defs, body.args[1], (__cps_v_24) => {
        return __k(RuleBody["pratt"](body.args[0], __cps_v_24));
      });
    }
  });
}

export function resolve_pratt_alts(__caps, token_defs, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const __match_91 = alts.args[0];
      return resolve_elements(__caps, token_defs, __match_91.args[1], (__cps_v_28) => {
        const __cps_v_26 = PrattAlt["mk"](__match_91.args[0], __cps_v_28, __match_91.args[2]);
        return resolve_pratt_alts(__caps, token_defs, alts.args[1], (__cps_v_27) => {
          return __k(List["cons"](__cps_v_26, __cps_v_27));
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
      return resolve_element(__caps, token_defs, elems.args[0], (__cps_v_29) => {
        return resolve_elements(__caps, token_defs, elems.args[1], (__cps_v_30) => {
          return __k(List["cons"](__cps_v_29, __cps_v_30));
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
        return resolve_element(__caps, token_defs, elem.args[1], (__cps_v_34) => {
          return __k(Element["labeled"](label, __cps_v_34));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_33) => {
          return __k(Element["optional"](__cps_v_33));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_32) => {
          return __k(Element["repeated"](__cps_v_32));
        });
      })(elem.args[0]) : ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_31) => {
          return __k(Element["group"](__cps_v_31));
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


export const List = { "nil": { [LUMO_TAG]: "nil" }, "cons": (arg0, arg1) => {
  return { [LUMO_TAG]: "cons", args: [arg0, arg1] };
} };

export const __impl_List = { map: () => {
  return __lumo_error();
} };


export const Ordering = { "less": { [LUMO_TAG]: "less" }, "equal": { [LUMO_TAG]: "equal" }, "greater": { [LUMO_TAG]: "greater" } };










export const __impl_Bool_Not = (__k_handle) => {
  return { not: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((__match_103) => {
        if (__match_103) {
          return false;
        } else {
          return true;
        }
      })(self)));
    });
  } };
};


export const __impl_Number_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.eq(__caps, self, other, (__cps_v_35) => {
        return __k_handle(__k_perform(__cps_v_35));
      });
    });
  } };
};

export const __impl_Number_PartialOrd = (__k_handle) => {
  return { cmp: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.cmp(__caps, self, other, (__cps_v_36) => {
        return __k_handle(__k_perform(__cps_v_36));
      });
    });
  } };
};

export const __impl_Number_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.add(__caps, self, other, (__cps_v_37) => {
        return __k_handle(__k_perform(__cps_v_37));
      });
    });
  } };
};

export const __impl_Number_Sub = (__k_handle) => {
  return { sub: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.sub(__caps, self, other, (__cps_v_38) => {
        return __k_handle(__k_perform(__cps_v_38));
      });
    });
  } };
};

export const __impl_Number_Mul = (__k_handle) => {
  return { mul: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mul(__caps, self, other, (__cps_v_39) => {
        return __k_handle(__k_perform(__cps_v_39));
      });
    });
  } };
};

export const __impl_Number_Div = (__k_handle) => {
  return { div: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.div(__caps, self, other, (__cps_v_40) => {
        return __k_handle(__k_perform(__cps_v_40));
      });
    });
  } };
};

export const __impl_Number_Mod = (__k_handle) => {
  return { mod_: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mod_(__caps, self, other, (__cps_v_41) => {
        return __k_handle(__k_perform(__cps_v_41));
      });
    });
  } };
};

export const __impl_Number_Neg = (__k_handle) => {
  return { neg: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.neg(__caps, self, (__cps_v_42) => {
        return __k_handle(__k_perform(__cps_v_42));
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
      return __k_handle(__k_perform(((__match_104) => {
        if (__match_104) {
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


export const __impl_String_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.concat(__caps, self, other, (__cps_v_43) => {
        return __k_handle(__k_perform(__cps_v_43));
      });
    });
  } };
};

export const __impl_String_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.eq(__caps, self, other, (__cps_v_44) => {
        return __k_handle(__k_perform(__cps_v_44));
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

export function collect_tokens_from_alts__lto_9309ae26(__caps, alts, kws, syms, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else {
      const rest = alts.args[1];
      const name = alts.args[0].args[0];
      const code = String.char_code_at(name, 0);
      const __match_115 = ((code < 65) ? Ordering["less"] : ((__match_114) => {
        if (__match_114) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(code, 65)));
      if (((__match_115[LUMO_TAG] === "less") ? false : ((__match_115[LUMO_TAG] === "equal") ? true : true))) {
        const __match_112 = ((code < 90) ? Ordering["less"] : ((__match_111) => {
          if (__match_111) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(code, 90)));
        if (((__match_112[LUMO_TAG] === "less") ? true : ((__match_112[LUMO_TAG] === "equal") ? true : false))) {
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
  const __lto_b_31 = String.len(s1);
  const __match_118 = ((idx < __lto_b_31) ? Ordering["less"] : ((__match_117) => {
    if (__match_117) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(idx, __lto_b_31)));
  if (((__match_118[LUMO_TAG] === "less") ? false : ((__match_118[LUMO_TAG] === "equal") ? true : true))) {
    let __match_135;
    let __match_134;
    const __lto_b_35 = String.len(s2);
    if ((idx < __lto_b_35)) {
      __match_134 = Ordering["less"];
    } else if ((idx === __lto_b_35)) {
      __match_134 = Ordering["equal"];
    } else {
      __match_134 = Ordering["greater"];
    }
    if ((__match_134[LUMO_TAG] === "less")) {
      __match_135 = false;
    } else if ((__match_134[LUMO_TAG] === "equal")) {
      __match_135 = true;
    } else {
      __match_135 = true;
    }
    if (__match_135) {
      return false;
    } else {
      return true;
    }
  } else {
    let __match_123;
    let __match_122;
    const __lto_b_39 = String.len(s2);
    if ((idx < __lto_b_39)) {
      __match_122 = Ordering["less"];
    } else if ((idx === __lto_b_39)) {
      __match_122 = Ordering["equal"];
    } else {
      __match_122 = Ordering["greater"];
    }
    if ((__match_122[LUMO_TAG] === "less")) {
      __match_123 = false;
    } else if ((__match_122[LUMO_TAG] === "equal")) {
      __match_123 = true;
    } else {
      __match_123 = true;
    }
    if (__match_123) {
      return false;
    } else {
      const ca = String.char_code_at(s1, idx);
      const cb = String.char_code_at(s2, idx);
      const __match_126 = ((ca < cb) ? Ordering["less"] : ((__match_125) => {
        if (__match_125) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(ca, cb)));
      if (((__match_126[LUMO_TAG] === "less") ? true : ((__match_126[LUMO_TAG] === "equal") ? false : false))) {
        return true;
      } else {
        let __match_131;
        let __match_130;
        if ((cb < ca)) {
          __match_130 = Ordering["less"];
        } else if ((cb === ca)) {
          __match_130 = Ordering["equal"];
        } else {
          __match_130 = Ordering["greater"];
        }
        if ((__match_130[LUMO_TAG] === "less")) {
          __match_131 = true;
        } else if ((__match_130[LUMO_TAG] === "equal")) {
          __match_131 = false;
        } else {
          __match_131 = false;
        }
        if (__match_131) {
          return false;
        } else {
          return string_lt_loop__lto_090deca7(s1, s2, ((__lto_self_48) => {
            return (__lto_self_48 + 1);
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
    const __match_141 = ((code < 65) ? Ordering["less"] : ((__match_140) => {
      if (__match_140) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_141[LUMO_TAG] === "less") ? false : ((__match_141[LUMO_TAG] === "equal") ? true : true)) ? ((__match_145) => {
      if ((__match_145[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_145[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_56) => {
      const __lto_other_57 = 90;
      const __match_143 = (__lto_self_56 < __lto_other_57);
      if (__match_143) {
        return Ordering["less"];
      } else {
        const __match_144 = (__lto_self_56 === __lto_other_57);
        if (__match_144) {
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
    return emit_variants_from_names__lto_1ba4622a(((__lto_self_148) => {
      return (__lto_self_148 + "<'a>),\n");
    })(((__lto_self_150) => {
      return (__lto_self_150 + name);
    })(((__lto_self_152) => {
      return (__lto_self_152 + "(");
    })(((__lto_self_154) => {
      return (__lto_self_154 + name);
    })(((__lto_self_156) => {
      return (__lto_self_156 + "    ");
    })(s))))), names.args[1]);
  }
}

export function emit_cast_chain_from_names__lto_1ba4622a(s, names) {
  if ((names[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = names.args[0];
    return emit_cast_chain_from_names__lto_1ba4622a(((__lto_self_184) => {
      return (__lto_self_184 + (((("            .or_else(|| " + name) + "::cast(node).map(Self::") + name) + "))\n"));
    })(s), names.args[1]);
  }
}

export function emit_syntax_arms_from_names__lto_1ba4622a(s, names) {
  if ((names[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return emit_syntax_arms_from_names__lto_1ba4622a(((__lto_self_196) => {
      return (__lto_self_196 + (("            Self::" + names.args[0]) + "(n) => n.syntax(),\n"));
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

export function emit_single_accessor_ctx__lto_3890158f(__caps, s, label, elem, token_defs, prev_kw, __k) {
  return __thunk(() => {
    if ((prev_kw === "")) {
      return emit_single_accessor(__caps, s, label, elem, token_defs, __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if (list_contains_string__lto_3890158f(token_defs, name)) {
        return emit_single_accessor(__caps, s, label, elem, token_defs, __k);
      } else {
        return keyword_variant__lto_1ba4622a(__caps, prev_kw, (__cps_v_45) => {
          return __k(emit_node_accessor_after_kw__lto_1ba4622a(s, label, name, __cps_v_45));
        });
      }
    } else if ((elem[LUMO_TAG] === "optional")) {
      return emit_single_accessor_ctx__lto_3890158f(__caps, s, label, elem.args[0], token_defs, prev_kw, __k);
    } else {
      return emit_single_accessor(__caps, s, label, elem, token_defs, __k);
    }
  });
}

export function emit_node_accessor_after_kw__lto_1ba4622a(s, label, node_name, kw_kind) {
  return ((((((((((((((((((((((s + "    pub fn ") + label) + "(&self) -> Option<") + node_name) + "<'a>> {\n") + "        let mut found_kw = false;\n") + "        for c in &self.0.children {\n") + "            if !found_kw {\n") + "                if let SyntaxElement::Token(t) = c {\n") + "                    if t.kind == SyntaxKind::") + kw_kind) + " { found_kw = true; }\n") + "                }\n") + "            } else {\n") + "                if let SyntaxElement::Node(n) = c {\n") + "                    if let Some(result) = ") + node_name) + "::cast(n) { return Some(result); }\n") + "                }\n") + "            }\n") + "        }\n") + "        None\n    }\n");
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
    return emit_enum_variants__lto_1ba4622a(((__lto_self_596) => {
      return (__lto_self_596 + "<'a>),\n");
    })(((__lto_self_598) => {
      return (__lto_self_598 + name);
    })(((__lto_self_600) => {
      return (__lto_self_600 + "(");
    })(((__lto_self_602) => {
      return (__lto_self_602 + name);
    })(((__lto_self_604) => {
      return (__lto_self_604 + "    ");
    })(s))))), alts.args[1]);
  }
}

export function emit_enum_cast_chain__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = alts.args[0].args[0];
    return emit_enum_cast_chain__lto_1ba4622a(((__lto_self_632) => {
      return (__lto_self_632 + (((("            .or_else(|| " + name) + "::cast(node).map(Self::") + name) + "))\n"));
    })(s), alts.args[1]);
  }
}

export function emit_enum_syntax_arms__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return emit_enum_syntax_arms__lto_1ba4622a(((__lto_self_644) => {
      return (__lto_self_644 + (("            Self::" + alts.args[0].args[0]) + "(n) => n.syntax(),\n"));
    })(s), alts.args[1]);
  }
}

export function emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return __k((((((((((((((s + "pub struct ") + name) + "<'a>(pub(crate) &'a SyntaxNode);\n\n") + "impl<'a> AstNode<'a> for ") + name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        (node.kind == SyntaxKind::") + kind) + ").then(|| Self(node))\n") + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n") + "}\n\n"));
  });
}

export function generate_parser__lto_1ba4622a(__caps, grammar, __k) {
  return __thunk(() => {
    const attrs = grammar.args[1];
    return collect_tokens(__caps, grammar, (collected) => {
      const _syms = collected.args[1];
      return ((keywords) => {
        return emit_parser_boilerplate__lto_1ba4622a(__caps, ((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n") + "#![allow(dead_code)]\n\n") + "use lumo_lexer::{lex_lossless, Keyword, LosslessTokenKind as LexKind};\n") + "use lumo_span::Span;\n\n") + "use crate::syntax_kind::SyntaxKind;\n\n"), keywords, (s) => {
          return emit_parser_impl__lto_1ba4622a(__caps, s, grammar.args[0], grammar.args[2], (s) => {
            return __k(s);
          });
        });
      })(collected.args[0]);
    });
  });
}

export function emit_parser_boilerplate__lto_1ba4622a(__caps, s, keywords, __k) {
  return __thunk(() => {
    return emit_lexer_kind_map__lto_1ba4622a(__caps, ((((((((((((((((((((((((((((((((((((((((s + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct ParseError {\n    pub span: Span,\n    pub message: String,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct LosslessToken {\n    pub kind: SyntaxKind,\n    pub span: Span,\n    pub text: String,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub enum SyntaxElement {\n    Node(Box<SyntaxNode>),\n    Token(LosslessToken),\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct SyntaxNode {\n    pub kind: SyntaxKind,\n    pub span: Span,\n    pub children: Vec<SyntaxElement>,\n}\n\n") + "#[derive(Debug, Clone, PartialEq, Eq)]\n") + "pub struct ParseOutput {\n    pub root: SyntaxNode,\n    pub errors: Vec<ParseError>,\n}\n\n") + "pub fn parse(source: &str) -> ParseOutput {\n") + "    let lexed = lex_lossless(source);\n") + "    let mut p = Parser { tokens: lexed.tokens, index: 0, errors: Vec::new() };\n") + "    let root = p.parse_file();\n") + "    ParseOutput { root, errors: p.errors }\n}\n\n") + "pub fn node_text(node: &SyntaxNode) -> String {\n") + "    let mut out = String::new();\n") + "    write_node_text(node, &mut out);\n") + "    out\n}\n\n") + "fn write_node_text(node: &SyntaxNode, out: &mut String) {\n") + "    for child in &node.children {\n") + "        match child {\n") + "            SyntaxElement::Node(n) => write_node_text(n, out),\n") + "            SyntaxElement::Token(t) => out.push_str(&t.text),\n") + "        }\n    }\n}\n\n") + "fn node_from_children(kind: SyntaxKind, children: Vec<SyntaxElement>) -> SyntaxNode {\n") + "    let span = children_span(&children);\n") + "    SyntaxNode { kind, span, children }\n}\n\n") + "fn children_span(children: &[SyntaxElement]) -> Span {\n") + "    let start = children.iter().find_map(|c| match c {\n") + "        SyntaxElement::Token(t) => Some(t.span.start),\n") + "        SyntaxElement::Node(n) => if n.children.is_empty() { None } else { Some(n.span.start) },\n") + "    }).unwrap_or(0);\n") + "    let end = children.iter().rev().find_map(|c| match c {\n") + "        SyntaxElement::Token(t) => Some(t.span.end),\n") + "        SyntaxElement::Node(n) => if n.children.is_empty() { None } else { Some(n.span.end) },\n") + "    }).unwrap_or(0);\n") + "    Span::new(start, end)\n}\n\n") + "fn lexer_token_to_lst(t: lumo_lexer::LosslessToken) -> LosslessToken {\n") + "    LosslessToken { kind: lexer_kind_to_syntax_kind(&t.kind, &t.text), span: t.span, text: t.text }\n}\n\n"), keywords, __k);
  });
}

export function emit_lexer_kind_map__lto_1ba4622a(__caps, s, keywords, __k) {
  return __thunk(() => {
    return emit_lexer_keyword_arms__lto_1ba4622a(__caps, (((((((((s + "fn lexer_kind_to_syntax_kind(kind: &LexKind, text: &str) -> SyntaxKind {\n") + "    match kind {\n") + "        LexKind::Ident => SyntaxKind::IDENT,\n") + "        LexKind::StringLit => SyntaxKind::STRING_LIT,\n") + "        LexKind::NumberLit => SyntaxKind::NUMBER_LIT,\n") + "        LexKind::Whitespace => SyntaxKind::WHITESPACE,\n") + "        LexKind::Newline => SyntaxKind::NEWLINE,\n") + "        LexKind::Unknown => SyntaxKind::UNKNOWN,\n") + "        LexKind::Keyword(kw) => match kw {\n"), keywords, (s) => {
      return __k(((((s + "            _ => SyntaxKind::UNKNOWN,\n") + "        },\n") + "        LexKind::Symbol(_) => SyntaxKind::from_symbol(text).unwrap_or(SyntaxKind::UNKNOWN),\n") + "    }\n}\n\n"));
    });
  });
}

export function emit_lexer_keyword_arms__lto_1ba4622a(__caps, s, keywords, __k) {
  return __thunk(() => {
    if ((keywords[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const kw = keywords.args[0];
      const rest = keywords.args[1];
      if (is_lexer_keyword__lto_3890158f(kw)) {
        const variant = keyword_variant_pascal__lto_3890158f(kw);
        return keyword_to_syntax_kind_name(__caps, kw, (kw_upper) => {
          return emit_lexer_keyword_arms__lto_1ba4622a(__caps, (((((s + "            Keyword::") + variant) + " => SyntaxKind::") + kw_upper) + ",\n"), rest, __k);
        });
      } else {
        return emit_lexer_keyword_arms__lto_1ba4622a(__caps, s, rest, __k);
      }
    }
  });
}

export function emit_parser_struct__lto_1ba4622a(s) {
  return (((((s + "struct Parser {\n") + "    tokens: Vec<lumo_lexer::LosslessToken>,\n") + "    index: usize,\n") + "    errors: Vec<ParseError>,\n") + "}\n\n");
}

export function emit_parser_helpers__lto_1ba4622a(s) {
  return (((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((s + "impl Parser {\n") + "    fn eof(&self) -> bool { self.index >= self.tokens.len() }\n") + "    fn current(&self) -> Option<&lumo_lexer::LosslessToken> { self.tokens.get(self.index) }\n") + "    fn bump(&mut self) -> Option<LosslessToken> {\n") + "        let token = self.tokens.get(self.index).cloned();\n") + "        if token.is_some() { self.index += 1; }\n") + "        token.map(lexer_token_to_lst)\n") + "    }\n") + "    fn is_trivia_lex(kind: &LexKind) -> bool {\n") + "        matches!(kind, LexKind::Whitespace | LexKind::Newline)\n") + "    }\n") + "    fn at_trivia(&self) -> bool {\n") + "        self.current().map(|t| Self::is_trivia_lex(&t.kind)).unwrap_or(false)\n") + "    }\n") + "    fn skip_trivia_into(&mut self, children: &mut Vec<SyntaxElement>) {\n") + "        while self.at_trivia() { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "    }\n") + "    fn peek_non_trivia_token(&self, n: usize) -> Option<&lumo_lexer::LosslessToken> {\n") + "        let mut count = 0; let mut i = self.index;\n") + "        while i < self.tokens.len() {\n") + "            let tok = &self.tokens[i];\n") + "            if !Self::is_trivia_lex(&tok.kind) {\n") + "                if count == n { return Some(tok); }\n") + "                count += 1;\n") + "            }\n") + "            i += 1;\n") + "        }\n") + "        None\n") + "    }\n") + "    fn at_keyword(&self, kw: Keyword) -> bool {\n") + "        matches!(self.current().map(|t| &t.kind), Some(LexKind::Keyword(actual)) if *actual == kw)\n") + "    }\n") + "    fn at_non_trivia_keyword(&self, kw: Keyword) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::Keyword(actual)) if *actual == kw)\n") + "    }\n") + "    fn at_non_trivia_symbol(&self, text: &str) -> bool {\n") + "        self.peek_non_trivia_token(0).map(|t| t.text.as_str()) == Some(text)\n") + "    }\n") + "    fn at_non_trivia_ident(&self) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::Ident))\n") + "    }\n") + "    fn at_non_trivia_string_lit(&self) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::StringLit))\n") + "    }\n") + "    fn at_non_trivia_number_lit(&self) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::NumberLit))\n") + "    }\n") + "    fn at_non_trivia_name(&self) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0).map(|t| &t.kind), Some(LexKind::Ident) | Some(LexKind::Keyword(_)))\n") + "    }\n") + "    fn expect_name(&mut self, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_non_trivia_name() { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(\"expected identifier\"); }\n") + "    }\n") + "    fn at_non_trivia_ident_text(&self, text: &str) -> bool {\n") + "        matches!(self.peek_non_trivia_token(0), Some(tok) if matches!(tok.kind, LexKind::Ident) && tok.text == text)\n") + "    }\n") + "    fn at_ident(&self) -> bool {\n") + "        matches!(self.current().map(|t| &t.kind), Some(LexKind::Ident))\n") + "    }\n") + "    fn at_symbol_text(&self, text: &str) -> bool {\n") + "        self.current().map(|t| t.text.as_str()) == Some(text)\n") + "    }\n") + "    fn at_trivia_or_unknown(&self) -> bool {\n") + "        self.current().map(|t| matches!(t.kind, LexKind::Whitespace | LexKind::Newline | LexKind::Unknown)).unwrap_or(false)\n") + "    }\n") + "    fn error_here(&mut self, message: &str) {\n") + "        let span = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));\n") + "        self.errors.push(ParseError { span, message: message.to_owned() });\n") + "    }\n") + "    fn expect_keyword(&mut self, kw: Keyword, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_keyword(kw) { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(\"expected keyword\"); }\n") + "    }\n") + "    fn expect_symbol(&mut self, sym: &str, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_symbol_text(sym) { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(&format!(\"expected '{}'\", sym)); }\n") + "    }\n") + "    fn expect_ident(&mut self, children: &mut Vec<SyntaxElement>) {\n") + "        self.skip_trivia_into(children);\n") + "        if self.at_ident() { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + "        else { self.error_here(\"expected identifier\"); }\n") + "    }\n") + "}\n\n");
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
    return to_snake(__caps, name, (__lto_other_1329) => {
      return make_body_lookahead(__caps, body, token_defs, (cond) => {
        return __k(((a, b) => {
          return (a + b);
        })(((((s + "    fn ") + ("can_parse_" + __lto_other_1329)) + "(&self) -> bool { ") + cond), " }\n"));
      });
    });
  });
}

export function make_first_elem_lookahead__lto_8227044e(__caps, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const elem = elems.args[0];
      const rest = elems.args[1];
      const __match_168 = unwrap_labeled_elem(elem);
      if ((__match_168[LUMO_TAG] === "optional")) {
        const inner = __match_168.args[0];
        return make_first_elem_lookahead__lto_8227044e(__caps, rest, token_defs, __k);
      } else if ((__match_168[LUMO_TAG] === "repeated")) {
        return make_first_elem_lookahead__lto_8227044e(__caps, rest, token_defs, (suffix) => {
          const attr_prefix = make_attr_star_prefix__lto_3890158f(__match_168.args[0]);
          if ((attr_prefix === "")) {
            return __k(suffix);
          } else if ((suffix === "false")) {
            return __k(attr_prefix);
          } else {
            return __k(((a, b) => {
              return (a + b);
            })((attr_prefix + " || "), suffix));
          }
        });
      } else {
        return make_element_lookahead__lto_8227044e(__caps, elem, token_defs, __k);
      }
    }
  });
}

export function make_attr_star_prefix__lto_3890158f(inner) {
  if ((inner[LUMO_TAG] === "node")) {
    if ((inner.args[0].args[0] === "Attribute")) {
      return "self.at_non_trivia_symbol(\"#\")";
    } else {
      return "";
    }
  } else {
    return "";
  }
}

export function make_alts_lookahead__lto_1ba4622a(__caps, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const rest = alts.args[1];
      return to_snake(__caps, alts.args[0].args[0], (__lto_other_1375) => {
        const cond = (("self.can_parse_" + __lto_other_1375) + "()");
        if ((rest[LUMO_TAG] === "nil")) {
          return __k(cond);
        } else {
          return make_alts_lookahead__lto_1ba4622a(__caps, rest, (__lto_other_1381) => {
            return __k(((a, b) => {
              return (a + b);
            })((cond + " || "), __lto_other_1381));
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
      const __match_180 = alts.args[0];
      const elems = __match_180.args[1];
      const __match_181 = __match_180.args[2];
      const rbp = __match_181.args[1];
      if ((__match_181.args[0][LUMO_TAG] === "num")) {
        return make_prefix_alts_lookahead__lto_8227044e(__caps, rest, __k);
      } else {
        return to_snake(__caps, __match_180.args[0], (__lto_other_1407) => {
          const cond = (("self.at_pratt_" + __lto_other_1407) + "()");
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
      return to_snake(__caps, atom_names.args[0], (__lto_other_1427) => {
        const cond = (("self.can_parse_" + __lto_other_1427) + "()");
        if ((rest[LUMO_TAG] === "nil")) {
          return __k(cond);
        } else {
          return make_atoms_lookahead__lto_1ba4622a(__caps, rest, (__lto_other_1433) => {
            return __k(((a, b) => {
              return (a + b);
            })((cond + " || "), __lto_other_1433));
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
      return to_snake(__caps, name, (__lto_other_1445) => {
        return to_screaming_snake(__caps, name, (kind) => {
          return emit_parse_elements(__caps, ((((s + "    fn ") + ("parse_" + __lto_other_1445)) + "(&mut self) -> SyntaxNode {\n") + "        let mut children = Vec::new();\n"), elems, token_defs, "        ", (s) => {
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
    return to_snake(__caps, name, (__lto_other_1537) => {
      return emit_alt_dispatch__lto_1ba4622a(__caps, (((s + "    fn ") + ("parse_" + __lto_other_1537)) + "(&mut self) -> SyntaxNode {\n"), alts, "        ", (s) => {
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
      return to_snake(__caps, alt_name, (__lto___lto_other_3591_3617) => {
        return to_snake(__caps, alt_name, (__lto_other_1575) => {
          return emit_alt_dispatch__lto_1ba4622a(__caps, ((((((s + indent) + "if ") + (("self.can_parse_" + __lto___lto_other_3591_3617) + "()")) + " { return ") + (("self.parse_" + __lto_other_1575) + "()")) + "; }\n"), alts.args[1], indent, __k);
        });
      });
    }
  });
}

export function emit_parse_pratt_rule__lto_1ba4622a(__caps, s, name, atom_names, alts, token_defs, __k) {
  return __thunk(() => {
    return to_snake(__caps, name, (__lto_other_1605) => {
      const fn_name = ("parse_" + __lto_other_1605);
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
      const __match_190 = alts.args[0];
      const name = __match_190.args[0];
      const elems = __match_190.args[1];
      const bp = __match_190.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, s, rest, token_defs, seen, __k);
      } else {
        return to_snake(__caps, name, (__lto_other_1685) => {
          return collect_pratt_cond_for_name__lto_8227044e(__caps, name, alts, token_defs, (cond) => {
            return emit_pratt_at_predicates_dedup__lto_1ba4622a(__caps, (((((s + "    fn ") + ("at_pratt_" + __lto_other_1685)) + "(&self) -> bool { ") + cond) + " }\n"), rest, token_defs, List["cons"](name, seen), __k);
          });
        });
      }
    }
  });
}

export function collect_pratt_cond_for_name__lto_8227044e(__caps, name, alts, token_defs, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const rest = alts.args[1];
      const __match_193 = alts.args[0];
      if ((__match_193.args[0] === name)) {
        return make_first_elem_lookahead__lto_8227044e(__caps, __match_193.args[1], token_defs, (cond) => {
          return collect_pratt_cond_for_name__lto_8227044e(__caps, name, rest, token_defs, (rest_cond) => {
            if ((rest_cond === "false")) {
              return __k(cond);
            } else {
              return __k(((a, b) => {
                return (a + b);
              })((cond + " || "), rest_cond));
            }
          });
        });
      } else {
        return collect_pratt_cond_for_name__lto_8227044e(__caps, name, rest, token_defs, __k);
      }
    }
  });
}

export function emit_pratt_loop__lto_1ba4622a(__caps, s, rule_name, alts, token_defs, indent, __k) {
  return __thunk(() => {
    return to_snake(__caps, rule_name, (__lto_other_1725) => {
      return emit_pratt_infix_alts__lto_1ba4622a(__caps, ((s + indent) + "loop {\n"), ("parse_" + __lto_other_1725), alts, token_defs, ((__lto_self_1736) => {
        return (__lto_self_1736 + "    ");
      })(indent), (s) => {
        return __k(((a, b) => {
          return (a + b);
        })((((s + indent) + "    break;\n") + indent), "}\n"));
      });
    });
  });
}

export function elem_has_expr__lto_3890158f(__caps, elem, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "node")) {
      return __k(((a, b) => {
        return (a === b);
      })(elem.args[0].args[0], "Expr"));
    } else if ((elem[LUMO_TAG] === "labeled")) {
      return elem_has_expr__lto_3890158f(__caps, elem.args[1], __k);
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return elem_has_expr__lto_3890158f(__caps, inner, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return elem_has_expr__lto_3890158f(__caps, inner, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "group") ? ((gelems) => {
        return elems_have_expr(__caps, gelems, __k);
      })(elem.args[0]) : __k(false))));
    }
  });
}

export function emit_pratt_infix_alts__lto_1ba4622a(__caps, s, fn_name, alts, token_defs, indent, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = alts.args[1];
      const __match_199 = alts.args[0];
      const name = __match_199.args[0];
      const elems = __match_199.args[1];
      const __match_200 = __match_199.args[2];
      const rbp = __match_200.args[1];
      const __match_201 = __match_200.args[0];
      if ((__match_201[LUMO_TAG] === "none")) {
        return emit_pratt_infix_alts__lto_1ba4622a(__caps, s, fn_name, rest, token_defs, indent, __k);
      } else {
        return to_screaming_snake(__caps, name, (kind) => {
          const lbp_str = Number.to_string(__match_201.args[0]);
          return make_first_elem_lookahead__lto_8227044e(__caps, elems, token_defs, (inline_cond) => {
            const __k_130 = (rbp_str) => {
              return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((((((((((s + indent) + "// ") + name) + " (lbp=") + lbp_str) + ")\n") + indent) + "if (") + inline_cond) + ") && ") + lbp_str) + " > min_bp {\n") + indent) + "    let mut children = vec![SyntaxElement::Node(Box::new(lhs))];\n"), elems, token_defs, fn_name, rbp_str, ((__lto_self_1816) => {
                return (__lto_self_1816 + "    ");
              })(indent), (sd) => {
                return elems_have_expr(__caps, elems, (__cps_v_46) => {
                  const __k_128 = (sd2) => {
                    return emit_pratt_infix_alts__lto_1ba4622a(__caps, ((((((((sd2 + indent) + "    lhs = node_from_children(SyntaxKind::") + kind) + ", children);\n") + indent) + "    continue;\n") + indent) + "}\n"), fn_name, rest, token_defs, indent, __k);
                  };
                  if (__cps_v_46) {
                    return __k_128(sd);
                  } else if ((rbp[LUMO_TAG] === "none")) {
                    return __k_128(sd);
                  } else {
                    return __k_128(((a, b) => {
                      return (a + b);
                    })((((((sd + indent) + "    children.push(SyntaxElement::Node(Box::new(self.") + fn_name) + "_bp(") + rbp_str), "))));\n"));
                  }
                });
              });
            };
            if ((rbp[LUMO_TAG] === "none")) {
              return __k_130(lbp_str);
            } else {
              return __k_130(Number.to_string(rbp.args[0]));
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
      const __match_206 = elems.args[0];
      if ((__match_206[LUMO_TAG] === "node")) {
        const rname = __match_206.args[0].args[0];
        if ((rname === "Expr")) {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.") + fn_name) + "_bp(") + rbp_str) + "))));\n"), rest, token_defs, fn_name, rbp_str, indent, __k);
        } else {
          return to_snake(__caps, rname, (__lto_other_1907) => {
            return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.parse_") + __lto_other_1907) + "())));\n"), rest, token_defs, fn_name, rbp_str, indent, __k);
          });
        }
      } else if ((__match_206[LUMO_TAG] === "labeled")) {
        const label = __match_206.args[0];
        return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](__match_206.args[1], rest), token_defs, fn_name, rbp_str, indent, __k);
      } else {
        return ((__match_206[LUMO_TAG] === "token") ? ((t) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, emit_parse_token_element__lto_8227044e(s, t, indent), rest, token_defs, fn_name, rbp_str, indent, __k);
        })(__match_206.args[0]) : ((__match_206[LUMO_TAG] === "optional") ? ((inner) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](inner, rest), token_defs, fn_name, rbp_str, indent, __k);
        })(__match_206.args[0]) : ((__match_206[LUMO_TAG] === "repeated") ? ((inner) => {
          return emit_parse_elements_filter_pratt__lto_8227044e(__caps, s, List["cons"](inner, rest), token_defs, fn_name, rbp_str, indent, __k);
        })(__match_206.args[0]) : ((gelems) => {
          return make_any_elems_lookahead__lto_1ba4622a(__caps, gelems, token_defs, (cond) => {
            return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if ") + cond) + " { children.push(SyntaxElement::Token(self.bump().unwrap())); }\n") + indent) + "else { self.error_here(\"expected operator\"); }\n"), rest, token_defs, fn_name, rbp_str, indent, __k);
          });
        })(__match_206.args[0]))));
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
      const __match_210 = alts.args[0];
      const name = __match_210.args[0];
      const __match_211 = __match_210.args[2];
      if ((__match_211.args[0][LUMO_TAG] === "num")) {
        return emit_prefix_alts__lto_1ba4622a(__caps, s, rule_name, rest, token_defs, indent, __k);
      } else {
        return to_screaming_snake(__caps, name, (kind) => {
          const __k_139 = (rbp_str) => {
            return to_snake(__caps, rule_name, (__lto_other_1969) => {
              return to_snake(__caps, name, (__lto_other_1975) => {
                return emit_parse_elements_filter_pratt__lto_8227044e(__caps, ((((((s + indent) + "if self.at_pratt_") + __lto_other_1975) + "() {\n") + indent) + "    let mut children = Vec::new();\n"), __match_210.args[1], token_defs, ("parse_" + __lto_other_1969), rbp_str, ((__lto_self_1996) => {
                  return (__lto_self_1996 + "    ");
                })(indent), (sc) => {
                  return emit_prefix_alts__lto_1ba4622a(__caps, ((((((sc + indent) + "    return node_from_children(SyntaxKind::") + kind) + ", children);\n") + indent) + "}\n"), rule_name, rest, token_defs, indent, __k);
                });
              });
            });
          };
          const __match_213 = __match_211.args[1];
          if ((__match_213[LUMO_TAG] === "none")) {
            return __k_139("0");
          } else {
            return __k_139(Number.to_string(__match_213.args[0]));
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
      return to_snake(__caps, name, (__lto_other_2027) => {
        return to_snake(__caps, name, (__lto_other_2035) => {
          return emit_atom_dispatch_alts__lto_1ba4622a(__caps, ((((((s + indent) + "if ") + (("self.can_parse_" + __lto_other_2027) + "()")) + " { ") + (("return self.parse_" + __lto_other_2035) + "();")) + " }\n"), atom_names.args[1], indent, __k);
        });
      });
    }
  });
}

export function emit_parse_element__lto_8227044e(__caps, s, elem, token_defs, indent, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      return __k(emit_parse_token_element__lto_8227044e(s, elem.args[0], indent));
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if ((name === "AttrName")) {
        return __k(((a, b) => {
          return (a + b);
        })((s + indent), "self.expect_name(&mut children);\n"));
      } else if (list_contains_string__lto_3890158f(token_defs, name)) {
        return __k(((a, b) => {
          return (a + b);
        })((((((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if matches!(self.current().map(|t| &t.kind), Some(LexKind::") + to_named_lex__lto_3890158f(name)) + ")) {\n") + indent) + "    children.push(SyntaxElement::Token(self.bump().unwrap()));\n") + indent) + "} else { self.error_here(\"expected ") + name), "\"); }\n"));
      } else {
        return to_snake(__caps, name, (__lto_other_2127) => {
          return __k(((a, b) => {
            return (a + b);
          })((((s + indent) + "children.push(SyntaxElement::Node(Box::new(self.parse_") + __lto_other_2127), "())));\n"));
        });
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        return emit_parse_element__lto_8227044e(__caps, s, elem.args[1], token_defs, indent, __k);
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

export function emit_parse_token_element__lto_8227044e(s, t, indent) {
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
    if ((name === "AttrName")) {
      return ((s + indent) + "self.expect_name(&mut children);\n");
    } else {
      return ((((((((((((s + indent) + "self.skip_trivia_into(&mut children);\n") + indent) + "if matches!(self.current().map(|t| &t.kind), Some(LexKind::") + to_named_lex__lto_3890158f(name)) + ")) {\n") + indent) + "    children.push(SyntaxElement::Token(self.bump().unwrap()));\n") + indent) + "} else { self.error_here(\"expected ") + name) + "\"); }\n");
    }
  }
}

export function emit_parse_optional__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k) {
  return make_element_lookahead__lto_8227044e(__caps, inner, token_defs, (cond) => {
    return emit_parse_element__lto_8227044e(__caps, ((((s + indent) + "if ") + cond) + " {\n"), inner, token_defs, ((__lto_self_2296) => {
      return (__lto_self_2296 + "    ");
    })(indent), (s2) => {
      return __k(((a, b) => {
        return (a + b);
      })((s2 + indent), "}\n"));
    });
  });
}

export function emit_parse_repeated__lto_1ba4622a(__caps, s, inner, token_defs, indent, __k) {
  return make_element_lookahead__lto_8227044e(__caps, inner, token_defs, (cond) => {
    return emit_parse_element__lto_8227044e(__caps, ((((s + indent) + "while ") + cond) + " {\n"), inner, token_defs, ((__lto_self_2324) => {
      return (__lto_self_2324 + "    ");
    })(indent), (s2) => {
      return __k(((a, b) => {
        return (a + b);
      })((s2 + indent), "}\n"));
    });
  });
}

export function make_element_lookahead__lto_8227044e(__caps, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const __match_226 = elem.args[0];
      if ((__match_226[LUMO_TAG] === "keyword")) {
        const kw = __match_226.args[0];
        if (is_lexer_keyword__lto_3890158f(kw)) {
          return __k(((a, b) => {
            return (a + b);
          })(("self.at_non_trivia_keyword(Keyword::" + keyword_variant_pascal__lto_3890158f(kw)), ")"));
        } else {
          return __k(((a, b) => {
            return (a + b);
          })(("self.at_non_trivia_ident_text(\"" + kw), "\")"));
        }
      } else if ((__match_226[LUMO_TAG] === "symbol")) {
        return __k(((a, b) => {
          return (a + b);
        })(("self.at_non_trivia_symbol(\"" + __match_226.args[0]), "\")"));
      } else {
        const name = __match_226.args[0];
        if ((name === "AttrName")) {
          return __k("self.at_non_trivia_name()");
        } else if ((name === "StringLit")) {
          return __k("self.at_non_trivia_string_lit()");
        } else if ((name === "NumberLit")) {
          return __k("self.at_non_trivia_number_lit()");
        } else {
          return __k("self.at_non_trivia_ident()");
        }
      }
    } else if ((elem[LUMO_TAG] === "node")) {
      const name = elem.args[0].args[0];
      if ((name === "AttrName")) {
        return __k("self.at_non_trivia_name()");
      } else if (list_contains_string__lto_3890158f(token_defs, name)) {
        return __k("self.at_non_trivia_ident()");
      } else {
        return to_snake(__caps, name, (__lto_other_2379) => {
          return __k(((a, b) => {
            return (a + b);
          })(("self.can_parse_" + __lto_other_2379), "()"));
        });
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        return make_element_lookahead__lto_8227044e(__caps, elem.args[1], token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return make_element_lookahead__lto_8227044e(__caps, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return make_element_lookahead__lto_8227044e(__caps, inner, token_defs, __k);
      })(elem.args[0]) : ((gelems) => {
        return make_group_lookahead(__caps, gelems, token_defs, __k);
      })(elem.args[0]))));
    }
  });
}

export function make_any_elems_lookahead__lto_1ba4622a(__caps, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k("false");
    } else {
      const rest = elems.args[1];
      return make_element_lookahead__lto_8227044e(__caps, elems.args[0], token_defs, (cond) => {
        if ((rest[LUMO_TAG] === "nil")) {
          return __k(cond);
        } else {
          return make_any_elems_lookahead__lto_1ba4622a(__caps, rest, token_defs, (__lto_other_2385) => {
            return __k(((a, b) => {
              return (a + b);
            })((cond + " || "), __lto_other_2385));
          });
        }
      });
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
  } else if ((kw === "type")) {
    return true;
  } else if ((kw === "produce")) {
    return true;
  } else if ((kw === "perform")) {
    return true;
  } else if ((kw === "lambda")) {
    return true;
  } else if ((kw === "roll")) {
    return true;
  } else if ((kw === "unroll")) {
    return true;
  } else if ((kw === "ctor")) {
    return true;
  } else if ((kw === "for")) {
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
  } else if ((kw === "type")) {
    return "Type";
  } else if ((kw === "produce")) {
    return "Produce";
  } else if ((kw === "perform")) {
    return "Perform";
  } else if ((kw === "lambda")) {
    return "Lambda";
  } else if ((kw === "roll")) {
    return "Roll";
  } else if ((kw === "unroll")) {
    return "Unroll";
  } else if ((kw === "ctor")) {
    return "Ctor";
  } else if ((kw === "for")) {
    return "For";
  } else {
    return kw;
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
      return to_screaming_snake(__caps, tokens.args[0], (__lto_other_2655) => {
        return emit_named_tokens__lto_1ba4622a(__caps, (((s + "    ") + __lto_other_2655) + ",\n"), tokens.args[1], __k);
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
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_2675) => {
        return emit_keywords_items__lto_1ba4622a(__caps, ((__lto_self_2684) => {
          return (__lto_self_2684 + (((("    " + __lto_other_2675) + ", // '") + kw) + "'\n"));
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
    return emit_symbols_items__lto_1ba4622a(((syms.args[1][LUMO_TAG] === "nil") ? s : ((__match_289) => {
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
    return emit_symbols_items__lto_1ba4622a(((__lto_self_2708) => {
      return (__lto_self_2708 + line);
    })(s), syms.args[1]);
  }
}

export function emit_node_kinds__lto_1ba4622a(__caps, s, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = rules.args[1];
      const __match_292 = rules.args[0];
      const name = __match_292.args[0];
      const __match_293 = __match_292.args[1];
      if ((__match_293[LUMO_TAG] === "sequence")) {
        const elems = __match_293.args[0];
        return to_screaming_snake(__caps, name, (__lto_other_2719) => {
          return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_2728) => {
            return (__lto_self_2728 + (((("    " + __lto_other_2719) + ", // ") + name) + "\n"));
          })(s), rest, __k);
        });
      } else if ((__match_293[LUMO_TAG] === "alternatives")) {
        if (is_token_only_alternatives__lto_9309ae26(__match_293.args[0])) {
          return to_screaming_snake(__caps, name, (__lto_other_2739) => {
            return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_2748) => {
              return (__lto_self_2748 + (((("    " + __lto_other_2739) + ", // ") + name) + " (token wrapper)\n"));
            })(s), rest, __k);
          });
        } else {
          return emit_node_kinds__lto_1ba4622a(__caps, s, rest, __k);
        }
      } else {
        const atom_names = __match_293.args[0];
        return emit_pratt_alt_kinds(__caps, s, __match_293.args[1], (s2) => {
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
      const __match_296 = alts.args[0];
      const name = __match_296.args[0];
      const elems = __match_296.args[1];
      const bp = __match_296.args[2];
      if (list_contains_string__lto_3890158f(seen, name)) {
        return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, s, rest, seen, __k);
      } else {
        return to_screaming_snake(__caps, name, (__lto_other_2759) => {
          return emit_pratt_alt_kinds_dedup__lto_1ba4622a(__caps, ((__lto_self_2768) => {
            return (__lto_self_2768 + (((("    " + __lto_other_2759) + ", // ") + name) + "\n"));
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
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_2795) => {
        return emit_keyword_arms__lto_1ba4622a(__caps, ((__lto_self_2808) => {
          return (__lto_self_2808 + (((("            \"" + kw) + "\" => Some(Self::") + __lto_other_2795) + "),\n"));
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
    return emit_symbol_arms__lto_1ba4622a(((__lto_self_2848) => {
      return (__lto_self_2848 + line);
    })(s), syms.args[1]);
  }
}

export function run__lto_3829b133(__caps, __k) {
  return __thunk(() => {
    const __lto_a_2854 = (__argv_length_raw() - 1);
    const __match_307 = ((__lto_a_2854 < 2) ? Ordering["less"] : ((__match_306) => {
      if (__match_306) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_a_2854, 2)));
    if (((__match_307[LUMO_TAG] === "less") ? true : ((__match_307[LUMO_TAG] === "equal") ? false : false))) {
      const __lto__err_2857 = __console_error("Usage: langue <input.langue> [output_dir]");
      return __k(__exit_process(1));
    } else {
      const file = __argv_at_raw(((__lto___lto_self_3598_3629) => {
        return (__lto___lto_self_3598_3629 + 1);
      })(1));
      return parse_grammar(__caps, readFileSync(file, "utf8"), (__cps_v_48) => {
        if ((__cps_v_48[LUMO_TAG] === "ok")) {
          return resolve_grammar(__caps, __cps_v_48.args[0], (grammar) => {
            const tokens = grammar.args[0];
            const count = list_length_rules__lto_92991de6(grammar.args[2]);
            return generate_syntax_kind__lto_1ba4622a(__caps, grammar, (syntax_kind_code) => {
              return generate_ast__lto_1ba4622a(__caps, grammar, (ast_code) => {
                return run_generate__lto_35421161(__caps, file, count, syntax_kind_code, ast_code, grammar, find_parser_path(grammar.args[1]), __k);
              });
            });
          });
        } else {
          const __lto__err_2861 = __console_error(((("Parse error at position " + Number.to_string(__cps_v_48.args[1])) + ": ") + __cps_v_48.args[0]));
          return __k(__exit_process(1));
        }
      });
    }
  });
}

export function run_generate__lto_35421161(__caps, file, count, syntax_kind_code, ast_code, grammar, parser_path, __k) {
  return __thunk(() => {
    const __lto_a_2876 = (__argv_length_raw() - 1);
    const __match_311 = ((__lto_a_2876 < 3) ? Ordering["less"] : ((__match_310) => {
      if (__match_310) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_a_2876, 3)));
    if (((__match_311[LUMO_TAG] === "less") ? true : ((__match_311[LUMO_TAG] === "equal") ? false : false))) {
      return write_output__lto_b8d7a8c4(__caps, ".", file, count, syntax_kind_code, ast_code, grammar, parser_path, __k);
    } else {
      return write_output__lto_b8d7a8c4(__caps, __argv_at_raw(((__lto___lto_self_3598_3638) => {
        return (__lto___lto_self_3598_3638 + 1);
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

export function to_screaming_snake_loop__lto_73ce111b(name, i, acc) {
  const __lto_b_2941 = String.len(name);
  const __match_316 = ((i < __lto_b_2941) ? Ordering["less"] : ((__match_315) => {
    if (__match_315) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_2941)));
  if (((__match_316[LUMO_TAG] === "less") ? false : ((__match_316[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __match_341 = ((code < 65) ? Ordering["less"] : ((__match_340) => {
      if (__match_340) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_341[LUMO_TAG] === "less") ? false : ((__match_341[LUMO_TAG] === "equal") ? true : true)) ? ((__match_345) => {
      if ((__match_345[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_345[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_2946) => {
      const __lto_other_2947 = 90;
      const __match_343 = (__lto_self_2946 < __lto_other_2947);
      if (__match_343) {
        return Ordering["less"];
      } else {
        const __match_344 = (__lto_self_2946 === __lto_other_2947);
        if (__match_344) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      let __match_322;
      let __match_321;
      if ((0 < i)) {
        __match_321 = Ordering["less"];
      } else if ((0 === i)) {
        __match_321 = Ordering["equal"];
      } else {
        __match_321 = Ordering["greater"];
      }
      if ((__match_321[LUMO_TAG] === "less")) {
        __match_322 = true;
      } else if ((__match_321[LUMO_TAG] === "equal")) {
        __match_322 = false;
      } else {
        __match_322 = false;
      }
      if (__match_322) {
        const prev_code = String.char_code_at(String.char_at(name, ((__lto_self_2954) => {
          return (__lto_self_2954 - 1);
        })(i)), 0);
        const __match_334 = ((prev_code < 97) ? Ordering["less"] : ((__match_333) => {
          if (__match_333) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 97)));
        const __match_327 = ((prev_code < 48) ? Ordering["less"] : ((__match_326) => {
          if (__match_326) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 48)));
        if ((((__match_334[LUMO_TAG] === "less") ? false : ((__match_334[LUMO_TAG] === "equal") ? true : true)) ? ((__match_338) => {
          if ((__match_338[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_338[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_2962) => {
          const __lto_other_2963 = 122;
          const __match_336 = (__lto_self_2962 < __lto_other_2963);
          if (__match_336) {
            return Ordering["less"];
          } else {
            const __match_337 = (__lto_self_2962 === __lto_other_2963);
            if (__match_337) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_2974) => {
            return (__lto_self_2974 + 1);
          })(i), ((__lto_self_2978) => {
            return (__lto_self_2978 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_2980) => {
            return (__lto_self_2980 + "_");
          })(acc)));
        } else if ((((__match_327[LUMO_TAG] === "less") ? false : ((__match_327[LUMO_TAG] === "equal") ? true : true)) ? ((__match_331) => {
          if ((__match_331[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_331[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_2970) => {
          const __lto_other_2971 = 57;
          const __match_329 = (__lto_self_2970 < __lto_other_2971);
          if (__match_329) {
            return Ordering["less"];
          } else {
            const __match_330 = (__lto_self_2970 === __lto_other_2971);
            if (__match_330) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_2986) => {
            return (__lto_self_2986 + 1);
          })(i), ((__lto_self_2990) => {
            return (__lto_self_2990 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_2992) => {
            return (__lto_self_2992 + "_");
          })(acc)));
        } else {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_2998) => {
            return (__lto_self_2998 + 1);
          })(i), ((__lto_self_3002) => {
            return (__lto_self_3002 + to_upper_char__lto_f0f5f7cb(c));
          })(acc));
        }
      } else {
        return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_3006) => {
          return (__lto_self_3006 + 1);
        })(i), ((__lto_self_3010) => {
          return (__lto_self_3010 + to_upper_char__lto_f0f5f7cb(c));
        })(acc));
      }
    } else {
      return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_3014) => {
        return (__lto_self_3014 + 1);
      })(i), ((__lto_self_3018) => {
        return (__lto_self_3018 + to_upper_char__lto_f0f5f7cb(c));
      })(acc));
    }
  }
}

export function to_upper_char__lto_f0f5f7cb(c) {
  const code = String.char_code_at(c, 0);
  const __match_348 = ((code < 97) ? Ordering["less"] : ((__match_347) => {
    if (__match_347) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_348[LUMO_TAG] === "less") ? false : ((__match_348[LUMO_TAG] === "equal") ? true : true))) {
    let __match_353;
    let __match_352;
    if ((code < 122)) {
      __match_352 = Ordering["less"];
    } else if ((code === 122)) {
      __match_352 = Ordering["equal"];
    } else {
      __match_352 = Ordering["greater"];
    }
    if ((__match_352[LUMO_TAG] === "less")) {
      __match_353 = true;
    } else if ((__match_352[LUMO_TAG] === "equal")) {
      __match_353 = true;
    } else {
      __match_353 = false;
    }
    if (__match_353) {
      return fromCharCode((code - 32));
    } else {
      return c;
    }
  } else {
    return c;
  }
}

export function keyword_variant__lto_1ba4622a(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__lto_self_3035) => {
    return __k(((a, b) => {
      return (a + b);
    })(__lto_self_3035, "_KW"));
  });
}

export function to_upper_string_loop__lto_1fab3ad0(s, i, acc) {
  const __lto_b_3042 = String.len(s);
  const __match_356 = ((i < __lto_b_3042) ? Ordering["less"] : ((__match_355) => {
    if (__match_355) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3042)));
  if (((__match_356[LUMO_TAG] === "less") ? false : ((__match_356[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    return to_upper_string_loop__lto_1fab3ad0(s, ((__lto_self_3043) => {
      return (__lto_self_3043 + 1);
    })(i), ((__lto_self_3047) => {
      return (__lto_self_3047 + to_upper_char__lto_f0f5f7cb(String.char_at(s, i)));
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

export function to_snake_loop__lto_1fab3ad0(name, i, acc) {
  const __lto_b_3178 = String.len(name);
  const __match_390 = ((i < __lto_b_3178) ? Ordering["less"] : ((__match_389) => {
    if (__match_389) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3178)));
  if (((__match_390[LUMO_TAG] === "less") ? false : ((__match_390[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __match_399 = ((code < 65) ? Ordering["less"] : ((__match_398) => {
      if (__match_398) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_399[LUMO_TAG] === "less") ? false : ((__match_399[LUMO_TAG] === "equal") ? true : true)) ? ((__match_403) => {
      if ((__match_403[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_403[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_3183) => {
      const __lto_other_3184 = 90;
      const __match_401 = (__lto_self_3183 < __lto_other_3184);
      if (__match_401) {
        return Ordering["less"];
      } else {
        const __match_402 = (__lto_self_3183 === __lto_other_3184);
        if (__match_402) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      let __match_396;
      let __match_395;
      if ((0 < i)) {
        __match_395 = Ordering["less"];
      } else if ((0 === i)) {
        __match_395 = Ordering["equal"];
      } else {
        __match_395 = Ordering["greater"];
      }
      if ((__match_395[LUMO_TAG] === "less")) {
        __match_396 = true;
      } else if ((__match_395[LUMO_TAG] === "equal")) {
        __match_396 = false;
      } else {
        __match_396 = false;
      }
      if (__match_396) {
        return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_3191) => {
          return (__lto_self_3191 + 1);
        })(i), ((__lto_self_3195) => {
          return (__lto_self_3195 + to_lower_char__lto_56361231(c));
        })(((__lto_self_3197) => {
          return (__lto_self_3197 + "_");
        })(acc)));
      } else {
        return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_3203) => {
          return (__lto_self_3203 + 1);
        })(i), ((__lto_self_3207) => {
          return (__lto_self_3207 + to_lower_char__lto_56361231(c));
        })(acc));
      }
    } else {
      return to_snake_loop__lto_1fab3ad0(name, ((__lto_self_3211) => {
        return (__lto_self_3211 + 1);
      })(i), ((__lto_self_3215) => {
        return (__lto_self_3215 + c);
      })(acc));
    }
  }
}

export function to_lower_char__lto_56361231(c) {
  const code = String.char_code_at(c, 0);
  const __match_406 = ((code < 65) ? Ordering["less"] : ((__match_405) => {
    if (__match_405) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 65)));
  if (((__match_406[LUMO_TAG] === "less") ? false : ((__match_406[LUMO_TAG] === "equal") ? true : true))) {
    let __match_411;
    let __match_410;
    if ((code < 90)) {
      __match_410 = Ordering["less"];
    } else if ((code === 90)) {
      __match_410 = Ordering["equal"];
    } else {
      __match_410 = Ordering["greater"];
    }
    if ((__match_410[LUMO_TAG] === "less")) {
      __match_411 = true;
    } else if ((__match_410[LUMO_TAG] === "equal")) {
      __match_411 = true;
    } else {
      __match_411 = false;
    }
    if (__match_411) {
      return fromCharCode((code + 32));
    } else {
      return c;
    }
  } else {
    return c;
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
  const __match_418 = ((code < 97) ? Ordering["less"] : ((__match_417) => {
    if (__match_417) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_418[LUMO_TAG] === "less") ? false : ((__match_418[LUMO_TAG] === "equal") ? true : true))) {
    let __match_429;
    if ((code < 122)) {
      __match_429 = Ordering["less"];
    } else if ((code === 122)) {
      __match_429 = Ordering["equal"];
    } else {
      __match_429 = Ordering["greater"];
    }
    if ((__match_429[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_429[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return false;
    }
  } else {
    let __match_423;
    let __match_422;
    if ((code < 65)) {
      __match_422 = Ordering["less"];
    } else if ((code === 65)) {
      __match_422 = Ordering["equal"];
    } else {
      __match_422 = Ordering["greater"];
    }
    if ((__match_422[LUMO_TAG] === "less")) {
      __match_423 = false;
    } else if ((__match_422[LUMO_TAG] === "equal")) {
      __match_423 = true;
    } else {
      __match_423 = true;
    }
    if (__match_423) {
      let __match_426;
      if ((code < 90)) {
        __match_426 = Ordering["less"];
      } else if ((code === 90)) {
        __match_426 = Ordering["equal"];
      } else {
        __match_426 = Ordering["greater"];
      }
      if ((__match_426[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_426[LUMO_TAG] === "equal")) {
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
  const __match_434 = ((code < 48) ? Ordering["less"] : ((__match_433) => {
    if (__match_433) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 48)));
  if (((__match_434[LUMO_TAG] === "less") ? false : ((__match_434[LUMO_TAG] === "equal") ? true : true))) {
    let __match_438;
    if ((code < 57)) {
      __match_438 = Ordering["less"];
    } else if ((code === 57)) {
      __match_438 = Ordering["equal"];
    } else {
      __match_438 = Ordering["greater"];
    }
    if ((__match_438[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_438[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return false;
    }
  } else {
    return false;
  }
}

export function state_eof__lto_9309ae26(st) {
  const __lto_a_3278 = st.args[1];
  const __lto_b_3279 = String.len(st.args[0]);
  const __match_442 = ((__lto_a_3278 < __lto_b_3279) ? Ordering["less"] : ((__match_441) => {
    if (__match_441) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_3278, __lto_b_3279)));
  if ((__match_442[LUMO_TAG] === "less")) {
    return false;
  } else if ((__match_442[LUMO_TAG] === "equal")) {
    return true;
  } else {
    return true;
  }
}

export function state_peek__lto_9309ae26(st) {
  const src = st.args[0];
  const pos = st.args[1];
  const __lto_b_3283 = String.len(src);
  const __match_446 = ((pos < __lto_b_3283) ? Ordering["less"] : ((__match_445) => {
    if (__match_445) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(pos, __lto_b_3283)));
  if (((__match_446[LUMO_TAG] === "less") ? true : ((__match_446[LUMO_TAG] === "equal") ? false : false))) {
    return String.char_at(src, pos);
  } else {
    return "";
  }
}

export function state_advance__lto_92991de6(st, n) {
  return ParseState["mk"](st.args[0], ((__lto_self_3284) => {
    return (__lto_self_3284 + n);
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
      const __lto_b_3299 = String.len(state_src(st));
      const __match_454 = ((next_pos < __lto_b_3299) ? Ordering["less"] : ((__match_453) => {
        if (__match_453) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(next_pos, __lto_b_3299)));
      if (((__match_454[LUMO_TAG] === "less") ? true : ((__match_454[LUMO_TAG] === "equal") ? false : false))) {
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
      return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_49) => {
        if (__cps_v_49) {
          const start = state_pos(st2);
          return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (end_st) => {
            return __k(ParseResult["ok"](String.slice(state_src(st2), start, state_pos(end_st)), end_st));
          });
        } else {
          return __k(ParseResult["err"](((__lto_self_3308) => {
            return (__lto_self_3308 + "'");
          })(((__lto_self_3310) => {
            return (__lto_self_3310 + state_peek__lto_9309ae26(st2));
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
  const __lto_a_3322 = (String.len(src) - pos);
  const __match_463 = ((__lto_a_3322 < len) ? Ordering["less"] : ((__match_462) => {
    if (__match_462) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_3322, len)));
  if (((__match_463[LUMO_TAG] === "less") ? false : ((__match_463[LUMO_TAG] === "equal") ? true : true))) {
    const slice = String.slice(src, pos, ((__lto_self_3324) => {
      return (__lto_self_3324 + len);
    })(pos));
    if ((slice === expected)) {
      return ParseResult["ok"](expected, state_advance__lto_92991de6(st2, len));
    } else {
      return ParseResult["err"](((__lto_self_3332) => {
        return (__lto_self_3332 + "'");
      })(((__lto_self_3334) => {
        return (__lto_self_3334 + slice);
      })(((__lto_self_3336) => {
        return (__lto_self_3336 + "', got '");
      })(((__lto_self_3338) => {
        return (__lto_self_3338 + expected);
      })("expected '")))), pos);
    }
  } else {
    return ParseResult["err"](((__lto_self_3348) => {
      return (__lto_self_3348 + "'");
    })(((__lto_self_3350) => {
      return (__lto_self_3350 + expected);
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
  const __lto_a_3374 = (pos + len);
  const __lto_b_3375 = String.len(src);
  const __match_471 = ((__lto_a_3374 < __lto_b_3375) ? Ordering["less"] : ((__match_470) => {
    if (__match_470) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_3374, __lto_b_3375)));
  if (((__match_471[LUMO_TAG] === "less") ? false : ((__match_471[LUMO_TAG] === "equal") ? false : true))) {
    return false;
  } else if ((String.slice(src, pos, ((__lto_self_3376) => {
    return (__lto_self_3376 + len);
  })(pos)) === word)) {
    let __match_477;
    let __match_476;
    const __lto_a_3390 = (pos + len);
    const __lto_b_3391 = String.len(src);
    if ((__lto_a_3390 < __lto_b_3391)) {
      __match_476 = Ordering["less"];
    } else if ((__lto_a_3390 === __lto_b_3391)) {
      __match_476 = Ordering["equal"];
    } else {
      __match_476 = Ordering["greater"];
    }
    if ((__match_476[LUMO_TAG] === "less")) {
      __match_477 = false;
    } else if ((__match_476[LUMO_TAG] === "equal")) {
      __match_477 = true;
    } else {
      __match_477 = true;
    }
    if (__match_477) {
      return true;
    } else if (is_ident_continue__lto_3890158f(String.char_at(src, ((__lto_self_3392) => {
      return (__lto_self_3392 + len);
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
    const __k_178 = (is_upper) => {
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
    const __match_486 = ((code < 65) ? Ordering["less"] : ((__match_485) => {
      if (__match_485) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if (((__match_486[LUMO_TAG] === "less") ? false : ((__match_486[LUMO_TAG] === "equal") ? true : true))) {
      const __match_481 = ((code < 90) ? Ordering["less"] : ((__match_483) => {
        if (__match_483) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(code, 90)));
      if ((__match_481[LUMO_TAG] === "less")) {
        return __k_178(true);
      } else if ((__match_481[LUMO_TAG] === "equal")) {
        return __k_178(true);
      } else {
        return __k_178(false);
      }
    } else {
      return __k_178(false);
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
    return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_50) => {
      if (__cps_v_50) {
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
  const __lto_b_3419 = String.len(s);
  const __match_491 = ((i < __lto_b_3419) ? Ordering["less"] : ((__match_490) => {
    if (__match_490) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3419)));
  if (((__match_491[LUMO_TAG] === "less") ? false : ((__match_491[LUMO_TAG] === "equal") ? true : true))) {
    return false;
  } else if (is_alpha__lto_9309ae26(String.char_at(s, i))) {
    return true;
  } else {
    return has_alpha__lto_090deca7(s, ((__lto_self_3420) => {
      return (__lto_self_3420 + 1);
    })(i));
  }
}

export function parse_grammar_items__lto_3890158f(__caps, st, tokens, attrs, rules, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_attr(attrs), list_reverse_rule(rules)), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "@")) {
      return parse_token_def(__caps, st2, (__cps_v_53) => {
        if ((__cps_v_53[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_53.args[1], list_concat_string(__cps_v_53.args[0], tokens), attrs, rules, __k);
        } else {
          return __k(ParseResult["err"](__cps_v_53.args[0], __cps_v_53.args[1]));
        }
      });
    } else if ((state_peek__lto_9309ae26(st2) === "#")) {
      return parse_grammar_attr__lto_8227044e(__caps, st2, (__cps_v_52) => {
        if ((__cps_v_52[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_52.args[1], tokens, List["cons"](__cps_v_52.args[0], attrs), rules, __k);
        } else {
          return __k(ParseResult["err"](__cps_v_52.args[0], __cps_v_52.args[1]));
        }
      });
    } else {
      return parse_rule(__caps, st2, (__cps_v_51) => {
        if ((__cps_v_51[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_51.args[1], tokens, attrs, List["cons"](__cps_v_51.args[0], rules), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_51.args[0], __cps_v_51.args[1]));
        }
      });
    }
  });
}

export function parse_grammar_attr__lto_8227044e(__caps, st, __k) {
  return __thunk(() => {
    const __match_500 = expect__lto_f3280589(st, "#");
    if ((__match_500[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_500.args[0], __match_500.args[1]));
    } else {
      const __match_501 = expect__lto_f3280589(__match_500.args[1], "[");
      if ((__match_501[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_501.args[0], __match_501.args[1]));
      } else {
        return parse_ident__lto_1ba4622a(__caps, __match_501.args[1], (__cps_v_55) => {
          if ((__cps_v_55[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_55.args[0], __cps_v_55.args[1]));
          } else {
            const attr_name = __cps_v_55.args[0];
            const st4 = __cps_v_55.args[1];
            if ((attr_name === "parser")) {
              const __match_504 = expect__lto_f3280589(st4, "(");
              if ((__match_504[LUMO_TAG] === "err")) {
                return __k(ParseResult["err"](__match_504.args[0], __match_504.args[1]));
              } else {
                return parse_parser_attr_args__lto_3890158f(__caps, __match_504.args[1], false, "", (__cps_v_54) => {
                  if ((__cps_v_54[LUMO_TAG] === "err")) {
                    return __k(ParseResult["err"](__cps_v_54.args[0], __cps_v_54.args[1]));
                  } else {
                    const __match_506 = expect__lto_f3280589(__cps_v_54.args[1], ")");
                    if ((__match_506[LUMO_TAG] === "err")) {
                      return __k(ParseResult["err"](__match_506.args[0], __match_506.args[1]));
                    } else {
                      const __match_507 = expect__lto_f3280589(__match_506.args[1], "]");
                      if ((__match_507[LUMO_TAG] === "err")) {
                        return __k(ParseResult["err"](__match_507.args[0], __match_507.args[1]));
                      } else {
                        return __k(ParseResult["ok"](GrammarAttr["parser_generate"](__cps_v_54.args[0]), __match_507.args[1]));
                      }
                    }
                  }
                });
              }
            } else {
              return __k(ParseResult["err"](((__lto_self_3436) => {
                return (__lto_self_3436 + attr_name);
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
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_57) => {
        if ((__cps_v_57[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_57.args[0], __cps_v_57.args[1]));
        } else {
          const __match_511 = expect__lto_f3280589(__cps_v_57.args[1], "=");
          if ((__match_511[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__match_511.args[0], __match_511.args[1]));
          } else {
            const st4 = __match_511.args[1];
            if ((__cps_v_57.args[0] === "path")) {
              const __match_514 = parse_string_lit__lto_38e07bea(st4);
              if ((__match_514[LUMO_TAG] === "err")) {
                return __k(ParseResult["err"](__match_514.args[0], __match_514.args[1]));
              } else {
                return parse_parser_attr_args__lto_3890158f(__caps, __match_514.args[1], true, __match_514.args[0], __k);
              }
            } else {
              return try_skip_attr_value__lto_3890158f(__caps, st4, (__cps_v_56) => {
                if ((__cps_v_56[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_56.args[0], __cps_v_56.args[1]));
                } else {
                  return parse_parser_attr_args__lto_3890158f(__caps, __cps_v_56.args[1], has_path, path, __k);
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
      const __match_517 = parse_string_lit__lto_38e07bea(st2);
      if ((__match_517[LUMO_TAG] === "ok")) {
        return __k(ParseResult["ok"]("", __match_517.args[1]));
      } else {
        return __k(ParseResult["err"](__match_517.args[0], __match_517.args[1]));
      }
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_58) => {
        if ((__cps_v_58[LUMO_TAG] === "ok")) {
          return __k(ParseResult["ok"]("", __cps_v_58.args[1]));
        } else {
          return __k(ParseResult["err"](__cps_v_58.args[0], __cps_v_58.args[1]));
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
      return peek_is_rule_start__lto_3890158f(__caps, st2, (__cps_v_61) => {
        if (__cps_v_61) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((state_peek__lto_9309ae26(st2) === "@")) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else {
          return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_60) => {
            if (__cps_v_60) {
              return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_59) => {
                if ((__cps_v_59[LUMO_TAG] === "ok")) {
                  return parse_token_names__lto_3890158f(__caps, __cps_v_59.args[1], List["cons"](__cps_v_59.args[0], acc), __k);
                } else {
                  return __k(ParseResult["err"](__cps_v_59.args[0], __cps_v_59.args[1]));
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
    return peek_is_pratt(__caps, st2, (__cps_v_63) => {
      if (__cps_v_63) {
        return parse_pratt_body(__caps, st2, __k);
      } else {
        return peek_char(__caps, st2, (__lto_self_3460) => {
          if ((__lto_self_3460 === "|")) {
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
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_66) => {
        if ((__cps_v_66[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_66.args[0], __cps_v_66.args[1]));
        } else {
          const name = __cps_v_66.args[0];
          const __match_528 = expect__lto_f3280589(__cps_v_66.args[1], ":");
          if ((__match_528[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__match_528.args[0], __match_528.args[1]));
          } else {
            const st4 = __match_528.args[1];
            if ((name === "atom")) {
              return parse_pratt_atom_list__lto_3890158f(__caps, st4, List["nil"], (__cps_v_65) => {
                if ((__cps_v_65[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_65.args[0], __cps_v_65.args[1]));
                } else {
                  return parse_pratt_items__lto_3890158f(__caps, __cps_v_65.args[1], list_concat_string(list_reverse_string(__cps_v_65.args[0]), atoms), alts, __k);
                }
              });
            } else {
              return parse_pratt_alt_body(__caps, st4, name, (__cps_v_64) => {
                if ((__cps_v_64[LUMO_TAG] === "err")) {
                  return __k(ParseResult["err"](__cps_v_64.args[0], __cps_v_64.args[1]));
                } else {
                  return parse_pratt_items__lto_3890158f(__caps, __cps_v_64.args[1], atoms, List["cons"](__cps_v_64.args[0], alts), __k);
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
      return peek_is_pratt_item_start__lto_94a384aa(__caps, st2, (__cps_v_70) => {
        if (__cps_v_70) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((state_peek__lto_9309ae26(st2) === "|")) {
          return parse_ident__lto_1ba4622a(__caps, state_advance__lto_92991de6(st2, 1), (__cps_v_69) => {
            if ((__cps_v_69[LUMO_TAG] === "ok")) {
              return parse_pratt_atom_list__lto_3890158f(__caps, __cps_v_69.args[1], List["cons"](__cps_v_69.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_69.args[0], __cps_v_69.args[1]));
            }
          });
        } else {
          return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_68) => {
            if (__cps_v_68) {
              return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_67) => {
                if ((__cps_v_67[LUMO_TAG] === "ok")) {
                  return parse_pratt_atom_list__lto_3890158f(__caps, __cps_v_67.args[1], List["cons"](__cps_v_67.args[0], acc), __k);
                } else {
                  return __k(ParseResult["err"](__cps_v_67.args[0], __cps_v_67.args[1]));
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
      return peek_is_pratt_item_start__lto_94a384aa(__caps, st2, (__cps_v_72) => {
        if (__cps_v_72) {
          return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
        } else if (peek_is_bp_marker__lto_3890158f(st2)) {
          return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
        } else {
          return parse_pratt_pattern_element__lto_3890158f(__caps, st2, (__cps_v_71) => {
            if ((__cps_v_71[LUMO_TAG] === "ok")) {
              return parse_pratt_pattern__lto_3890158f(__caps, __cps_v_71.args[1], List["cons"](__cps_v_71.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_71.args[0], __cps_v_71.args[1]));
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
      const __match_551 = parse_quoted__lto_38e07bea(st2);
      if ((__match_551[LUMO_TAG] === "ok")) {
        return classify_literal(__caps, __match_551.args[0], (__cps_v_77) => {
          return __k(apply_postfix_elem__lto_3890158f(Element["token"](__cps_v_77), __match_551.args[1]));
        });
      } else {
        return __k(ParseResult["err"](__match_551.args[0], __match_551.args[1]));
      }
    } else if ((state_peek__lto_9309ae26(st2) === "(")) {
      return parse_pratt_group__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), List["nil"], (__cps_v_75) => {
        if ((__cps_v_75[LUMO_TAG] === "ok")) {
          const __match_550 = expect__lto_f3280589(__cps_v_75.args[1], ")");
          if ((__match_550[LUMO_TAG] === "ok")) {
            return __k(apply_postfix_elem__lto_3890158f(Element["group"](__cps_v_75.args[0]), __match_550.args[1]));
          } else {
            return __k(ParseResult["err"](__match_550.args[0], __match_550.args[1]));
          }
        } else {
          return __k(ParseResult["err"](__cps_v_75.args[0], __cps_v_75.args[1]));
        }
      });
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_74) => {
        if ((__cps_v_74[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_74.args[0], __cps_v_74.args[1]));
        } else {
          const name = __cps_v_74.args[0];
          const st3 = __cps_v_74.args[1];
          if ((state_peek__lto_9309ae26(st3) === ":")) {
            return parse_pratt_pattern_element__lto_3890158f(__caps, state_advance__lto_92991de6(st3, 1), (__cps_v_73) => {
              if ((__cps_v_73[LUMO_TAG] === "ok")) {
                return __k(apply_postfix_elem__lto_3890158f(Element["labeled"](name, __cps_v_73.args[0]), __cps_v_73.args[1]));
              } else {
                return __k(ParseResult["err"](__cps_v_73.args[0], __cps_v_73.args[1]));
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
      return parse_pratt_pattern_element__lto_3890158f(__caps, st2, (__cps_v_78) => {
        if ((__cps_v_78[LUMO_TAG] === "ok")) {
          return parse_pratt_group__lto_3890158f(__caps, __cps_v_78.args[1], List["cons"](__cps_v_78.args[0], acc), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_78.args[0], __cps_v_78.args[1]));
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
      return __k(ParseResult["err"](((__lto_self_3504) => {
        return (__lto_self_3504 + "'");
      })(((__lto_self_3506) => {
        return (__lto_self_3506 + state_peek__lto_9309ae26(st2));
      })("expected number, got '")), state_pos(st2)));
    }
  });
}

export function parse_int__lto_1856fa45(s, i, acc) {
  const __lto_b_3515 = String.len(s);
  const __match_559 = ((i < __lto_b_3515) ? Ordering["less"] : ((__match_558) => {
    if (__match_558) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3515)));
  if (((__match_559[LUMO_TAG] === "less") ? false : ((__match_559[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const digit = (String.char_code_at(s, i) - 48);
    return parse_int__lto_1856fa45(s, ((__lto_self_3520) => {
      return (__lto_self_3520 + 1);
    })(i), ((__lto_self_3524) => {
      return (__lto_self_3524 + digit);
    })(((__lto_self_3526) => {
      return (__lto_self_3526 * 10);
    })(acc)));
  }
}

export function parse_alt_items__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_char(__caps, st2, (__lto_self_3532) => {
      if ((__lto_self_3532 === "|")) {
        const st3 = state_advance__lto_92991de6(skip_ws__lto_1bb67705(st2), 1);
        const st4 = skip_ws__lto_1bb67705(st3);
        if ((state_peek__lto_9309ae26(st4) === "'")) {
          const __match_564 = parse_quoted__lto_38e07bea(st4);
          if ((__match_564[LUMO_TAG] === "ok")) {
            return parse_alt_items__lto_3890158f(__caps, __match_564.args[1], List["cons"](Alternative["mk"](__match_564.args[0]), acc), __k);
          } else {
            return __k(ParseResult["err"](__match_564.args[0], __match_564.args[1]));
          }
        } else {
          return parse_ident__lto_1ba4622a(__caps, st3, (__cps_v_79) => {
            if ((__cps_v_79[LUMO_TAG] === "ok")) {
              return parse_alt_items__lto_3890158f(__caps, __cps_v_79.args[1], List["cons"](Alternative["mk"](__cps_v_79.args[0]), acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_79.args[0], __cps_v_79.args[1]));
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
      return peek_is_rule_start__lto_3890158f(__caps, st, (__cps_v_81) => {
        if (__cps_v_81) {
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
      const __match_578 = parse_quoted__lto_38e07bea(st2);
      if ((__match_578[LUMO_TAG] === "ok")) {
        return classify_literal(__caps, __match_578.args[0], (__cps_v_86) => {
          return __k(ParseResult["ok"](Element["token"](__cps_v_86), __match_578.args[1]));
        });
      } else {
        return __k(ParseResult["err"](__match_578.args[0], __match_578.args[1]));
      }
    } else if ((state_peek__lto_9309ae26(st2) === "(")) {
      return parse_group_elements__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), List["nil"], (__cps_v_84) => {
        if ((__cps_v_84[LUMO_TAG] === "ok")) {
          const __match_577 = expect__lto_f3280589(__cps_v_84.args[1], ")");
          if ((__match_577[LUMO_TAG] === "ok")) {
            return __k(ParseResult["ok"](Element["group"](__cps_v_84.args[0]), __match_577.args[1]));
          } else {
            return __k(ParseResult["err"](__match_577.args[0], __match_577.args[1]));
          }
        } else {
          return __k(ParseResult["err"](__cps_v_84.args[0], __cps_v_84.args[1]));
        }
      });
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_83) => {
        if ((__cps_v_83[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_83.args[0], __cps_v_83.args[1]));
        } else {
          const name = __cps_v_83.args[0];
          const st3 = __cps_v_83.args[1];
          if ((state_peek__lto_9309ae26(st3) === ":")) {
            return parse_element(__caps, state_advance__lto_92991de6(st3, 1), (__cps_v_82) => {
              if ((__cps_v_82[LUMO_TAG] === "ok")) {
                return __k(ParseResult["ok"](Element["labeled"](name, __cps_v_82.args[0]), __cps_v_82.args[1]));
              } else {
                return __k(ParseResult["err"](__cps_v_82.args[0], __cps_v_82.args[1]));
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
      return parse_element(__caps, st2, (__cps_v_87) => {
        if ((__cps_v_87[LUMO_TAG] === "ok")) {
          return parse_group_elements__lto_3890158f(__caps, __cps_v_87.args[1], List["cons"](__cps_v_87.args[0], acc), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_87.args[0], __cps_v_87.args[1]));
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
