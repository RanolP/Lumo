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
    return collect_tokens_from_rules(__caps, grammar.args[1], List["nil"], List["nil"], (pair) => {
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
    } else {
      return collect_tokens_from_alts__lto_9309ae26(__caps, body.args[0], kws, syms, __k);
    }
  });
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
    const __match_8 = collect_tokens_from_element(elems.args[0], kws, syms);
    return collect_tokens_from_elements(elems.args[1], __match_8.args[0], __match_8.args[1]);
  }
}

export function collect_tokens_from_element(elem, kws, syms) {
  if ((elem[LUMO_TAG] === "token")) {
    const __match_10 = elem.args[0];
    if ((__match_10[LUMO_TAG] === "keyword")) {
      return StringPair["mk"](List["cons"](__match_10.args[0], kws), syms);
    } else if ((__match_10[LUMO_TAG] === "symbol")) {
      return StringPair["mk"](kws, List["cons"](__match_10.args[0], syms));
    } else {
      const n = __match_10.args[0];
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

export function string_lt(__caps, a, b, __k) {
  return __thunk(() => {
    return __k(string_lt_loop__lto_090deca7(a, b, 0));
  });
}

export function emit_ast_rules(__caps, s, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __match_17 = rules.args[0];
      const name = __match_17.args[0];
      const __k_14 = (s2) => {
        return emit_ast_rules(__caps, s2, token_defs, rules.args[1], __k);
      };
      const __match_18 = __match_17.args[1];
      if ((__match_18[LUMO_TAG] === "sequence")) {
        return emit_struct_node__lto_1ba4622a(__caps, s, name, __match_18.args[0], token_defs, __k_14);
      } else {
        const alts = __match_18.args[0];
        if (is_token_only_alternatives__lto_9309ae26(alts)) {
          return emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k_14);
        } else {
          return __k_14(emit_enum_node__lto_1ba4622a(s, name, alts));
        }
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
      const __match_23 = elems.args[0];
      if ((__match_23[LUMO_TAG] === "labeled")) {
        return emit_single_accessor(__caps, s, __match_23.args[0], __match_23.args[1], token_defs, (s2) => {
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


export const Grammar = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const Rule = { "mk": (arg0, arg1) => {
  return { [LUMO_TAG]: "mk", args: [arg0, arg1] };
} };


export const RuleBody = { "sequence": (arg0) => {
  return { [LUMO_TAG]: "sequence", args: [arg0] };
}, "alternatives": (arg0) => {
  return { [LUMO_TAG]: "alternatives", args: [arg0] };
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
  return __trampoline(__main_cps({ IO_IO: IO(__identity), FS_FS: FS(__identity), Process_Process: Process(__identity), StrOps_StrOps: StrOps(__identity), Add_String: __impl_String_Add(__identity), Sub_Number: __impl_Number_Sub(__identity), NumOps_NumOps: NumOps(__identity), Add_Number: __impl_Number_Add(__identity), PartialEq_String: __impl_String_PartialEq(__identity), PartialOrd_Number: __impl_Number_PartialOrd(__identity) }, __identity));
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
    return parse_grammar_items__lto_3890158f(__caps, ParseState["mk"](src, 0), List["nil"], List["nil"], __k);
  });
}

export function parse_token_def(__caps, st, __k) {
  return __thunk(() => {
    const __match_36 = expect__lto_f3280589(st, "@token");
    if ((__match_36[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__match_36.args[0], __match_36.args[1]));
    } else {
      return parse_token_names__lto_3890158f(__caps, __match_36.args[1], List["nil"], __k);
    }
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident__lto_1ba4622a(__caps, st, (__cps_v_8) => {
    if ((__cps_v_8[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_8.args[0], __cps_v_8.args[1]));
    } else {
      const name = __cps_v_8.args[0];
      const __match_38 = expect__lto_f3280589(__cps_v_8.args[1], "=");
      if ((__match_38[LUMO_TAG] === "err")) {
        return __k(ParseResult["err"](__match_38.args[0], __match_38.args[1]));
      } else {
        return parse_rule_body__lto_3890158f(__caps, __match_38.args[1], name, (__cps_v_7) => {
          if ((__cps_v_7[LUMO_TAG] === "err")) {
            return __k(ParseResult["err"](__cps_v_7.args[0], __cps_v_7.args[1]));
          } else {
            return __k(ParseResult["ok"](Rule["mk"](name, __cps_v_7.args[0]), __cps_v_7.args[1]));
          }
        });
      }
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
      return is_seq_terminator__lto_3890158f(__caps, st2, (__cps_v_10) => {
        if (__cps_v_10) {
          return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
        } else {
          return parse_element(__caps, st2, (__cps_v_9) => {
            if ((__cps_v_9[LUMO_TAG] === "ok")) {
              return parse_seq_elements(__caps, __cps_v_9.args[1], List["cons"](__cps_v_9.args[0], acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_9.args[0], __cps_v_9.args[1]));
            }
          });
        }
      });
    }
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom__lto_3890158f(__caps, st, (__cps_v_11) => {
    if ((__cps_v_11[LUMO_TAG] === "err")) {
      return __k(ParseResult["err"](__cps_v_11.args[0], __cps_v_11.args[1]));
    } else {
      return __k(apply_postfix_elem__lto_3890158f(__cps_v_11.args[0], __cps_v_11.args[1]));
    }
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    const token_defs = g.args[0];
    return resolve_rules(__caps, token_defs, g.args[1], (__cps_v_12) => {
      return __k(Grammar["mk"](token_defs, __cps_v_12));
    });
  });
}

export function resolve_rules(__caps, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      const __match_46 = rules.args[0];
      return resolve_body(__caps, token_defs, __match_46.args[1], (resolved_body) => {
        return resolve_rules(__caps, token_defs, rules.args[1], (__cps_v_13) => {
          return __k(List["cons"](Rule["mk"](__match_46.args[0], resolved_body), __cps_v_13));
        });
      });
    }
  });
}

export function resolve_body(__caps, token_defs, body, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      return resolve_elements(__caps, token_defs, body.args[0], (__cps_v_14) => {
        return __k(RuleBody["sequence"](__cps_v_14));
      });
    } else {
      const alts = body.args[0];
      return __k(body);
    }
  });
}

export function resolve_elements(__caps, token_defs, elems, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else {
      return resolve_element(__caps, token_defs, elems.args[0], (__cps_v_15) => {
        return resolve_elements(__caps, token_defs, elems.args[1], (__cps_v_16) => {
          return __k(List["cons"](__cps_v_15, __cps_v_16));
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
        return resolve_element(__caps, token_defs, elem.args[1], (__cps_v_20) => {
          return __k(Element["labeled"](label, __cps_v_20));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_19) => {
          return __k(Element["optional"](__cps_v_19));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_18) => {
          return __k(Element["repeated"](__cps_v_18));
        });
      })(elem.args[0]) : ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_17) => {
          return __k(Element["group"](__cps_v_17));
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

export function list_concat_string(xs, ys) {
  if ((xs[LUMO_TAG] === "nil")) {
    return ys;
  } else {
    return List["cons"](xs.args[0], list_concat_string(xs.args[1], ys));
  }
}


export const Bool = { "true": true, "false": false };


export const __impl_String_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.concat(__caps, self, other, (__cps_v_21) => {
        return __k_handle(__k_perform(__cps_v_21));
      });
    });
  } };
};

export const __impl_String_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.eq(__caps, self, other, (__cps_v_22) => {
        return __k_handle(__k_perform(__cps_v_22));
      });
    });
  } };
};

export const String = { len: (__caps, self, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.len(__caps, self, __k);
  });
}, char_at: (__caps, self, idx, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.char_at(__caps, self, idx, __k);
  });
}, slice: (__caps, self, start, end, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.slice(__caps, self, start, end, __k);
  });
}, starts_with: (__caps, self, prefix, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.starts_with(__caps, self, prefix, __k);
  });
}, contains: (__caps, self, sub, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.contains(__caps, self, sub, __k);
  });
}, index_of: (__caps, self, sub, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.index_of(__caps, self, sub, __k);
  });
}, trim: (__caps, self, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.trim(__caps, self, __k);
  });
}, char_code_at: (__caps, self, idx, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.char_code_at(__caps, self, idx, __k);
  });
}, replace_all: (__caps, self, from, to, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.replace_all(__caps, self, from, to, __k);
  });
} };

export const Number = { to_string: (__caps, self, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.num_to_string(__caps, self, __k);
  });
}, to_char: (__caps, self, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.from_char_code(__caps, self, __k);
  });
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
      return __k_handle(__k_perform(((__match_57) => {
        if (__match_57) {
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
      return __caps.NumOps_NumOps.eq(__caps, self, other, (__cps_v_23) => {
        return __k_handle(__k_perform(__cps_v_23));
      });
    });
  } };
};

export const __impl_Number_PartialOrd = (__k_handle) => {
  return { cmp: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.cmp(__caps, self, other, (__cps_v_24) => {
        return __k_handle(__k_perform(__cps_v_24));
      });
    });
  } };
};

export const __impl_Number_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.add(__caps, self, other, (__cps_v_25) => {
        return __k_handle(__k_perform(__cps_v_25));
      });
    });
  } };
};

export const __impl_Number_Sub = (__k_handle) => {
  return { sub: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.sub(__caps, self, other, (__cps_v_26) => {
        return __k_handle(__k_perform(__cps_v_26));
      });
    });
  } };
};

export const __impl_Number_Mul = (__k_handle) => {
  return { mul: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mul(__caps, self, other, (__cps_v_27) => {
        return __k_handle(__k_perform(__cps_v_27));
      });
    });
  } };
};

export const __impl_Number_Div = (__k_handle) => {
  return { div: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.div(__caps, self, other, (__cps_v_28) => {
        return __k_handle(__k_perform(__cps_v_28));
      });
    });
  } };
};

export const __impl_Number_Mod = (__k_handle) => {
  return { mod_: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mod_(__caps, self, other, (__cps_v_29) => {
        return __k_handle(__k_perform(__cps_v_29));
      });
    });
  } };
};

export const __impl_Number_Neg = (__k_handle) => {
  return { neg: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.neg(__caps, self, (__cps_v_30) => {
        return __k_handle(__k_perform(__cps_v_30));
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
      return __k_handle(__k_perform(((__match_58) => {
        if (__match_58) {
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
  const __lto_b_3 = String.len(name);
  const __match_62 = ((i < __lto_b_3) ? Ordering["less"] : ((__match_61) => {
    if (__match_61) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_3)));
  if (((__match_62[LUMO_TAG] === "less") ? false : ((__match_62[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __match_87 = ((code < 65) ? Ordering["less"] : ((__match_86) => {
      if (__match_86) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_87[LUMO_TAG] === "less") ? false : ((__match_87[LUMO_TAG] === "equal") ? true : true)) ? ((__match_91) => {
      if ((__match_91[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_91[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_8) => {
      const __lto_other_9 = 90;
      const __match_89 = (__lto_self_8 < __lto_other_9);
      if (__match_89) {
        return Ordering["less"];
      } else {
        const __match_90 = (__lto_self_8 === __lto_other_9);
        if (__match_90) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      }
    })(code)) : false)) {
      let __match_68;
      let __match_67;
      if ((0 < i)) {
        __match_67 = Ordering["less"];
      } else if ((0 === i)) {
        __match_67 = Ordering["equal"];
      } else {
        __match_67 = Ordering["greater"];
      }
      if ((__match_67[LUMO_TAG] === "less")) {
        __match_68 = true;
      } else if ((__match_67[LUMO_TAG] === "equal")) {
        __match_68 = false;
      } else {
        __match_68 = false;
      }
      if (__match_68) {
        const prev_code = String.char_code_at(String.char_at(name, ((__lto_self_16) => {
          return (__lto_self_16 - 1);
        })(i)), 0);
        const __match_80 = ((prev_code < 97) ? Ordering["less"] : ((__match_79) => {
          if (__match_79) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 97)));
        const __match_73 = ((prev_code < 48) ? Ordering["less"] : ((__match_72) => {
          if (__match_72) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, 48)));
        if ((((__match_80[LUMO_TAG] === "less") ? false : ((__match_80[LUMO_TAG] === "equal") ? true : true)) ? ((__match_84) => {
          if ((__match_84[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_84[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_24) => {
          const __lto_other_25 = 122;
          const __match_82 = (__lto_self_24 < __lto_other_25);
          if (__match_82) {
            return Ordering["less"];
          } else {
            const __match_83 = (__lto_self_24 === __lto_other_25);
            if (__match_83) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_36) => {
            return (__lto_self_36 + 1);
          })(i), ((__lto_self_40) => {
            return (__lto_self_40 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_42) => {
            return (__lto_self_42 + "_");
          })(acc)));
        } else if ((((__match_73[LUMO_TAG] === "less") ? false : ((__match_73[LUMO_TAG] === "equal") ? true : true)) ? ((__match_77) => {
          if ((__match_77[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_77[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return false;
          }
        })(((__lto_self_32) => {
          const __lto_other_33 = 57;
          const __match_75 = (__lto_self_32 < __lto_other_33);
          if (__match_75) {
            return Ordering["less"];
          } else {
            const __match_76 = (__lto_self_32 === __lto_other_33);
            if (__match_76) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          }
        })(prev_code)) : false)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_48) => {
            return (__lto_self_48 + 1);
          })(i), ((__lto_self_52) => {
            return (__lto_self_52 + to_upper_char__lto_f0f5f7cb(c));
          })(((__lto_self_54) => {
            return (__lto_self_54 + "_");
          })(acc)));
        } else {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_60) => {
            return (__lto_self_60 + 1);
          })(i), ((__lto_self_64) => {
            return (__lto_self_64 + to_upper_char__lto_f0f5f7cb(c));
          })(acc));
        }
      } else {
        return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_68) => {
          return (__lto_self_68 + 1);
        })(i), ((__lto_self_72) => {
          return (__lto_self_72 + to_upper_char__lto_f0f5f7cb(c));
        })(acc));
      }
    } else {
      return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_76) => {
        return (__lto_self_76 + 1);
      })(i), ((__lto_self_80) => {
        return (__lto_self_80 + to_upper_char__lto_f0f5f7cb(c));
      })(acc));
    }
  }
}

export function to_upper_char__lto_f0f5f7cb(c) {
  const code = String.char_code_at(c, 0);
  const __match_94 = ((code < 97) ? Ordering["less"] : ((__match_93) => {
    if (__match_93) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_94[LUMO_TAG] === "less") ? false : ((__match_94[LUMO_TAG] === "equal") ? true : true))) {
    let __match_99;
    let __match_98;
    if ((code < 122)) {
      __match_98 = Ordering["less"];
    } else if ((code === 122)) {
      __match_98 = Ordering["equal"];
    } else {
      __match_98 = Ordering["greater"];
    }
    if ((__match_98[LUMO_TAG] === "less")) {
      __match_99 = true;
    } else if ((__match_98[LUMO_TAG] === "equal")) {
      __match_99 = true;
    } else {
      __match_99 = false;
    }
    if (__match_99) {
      return fromCharCode((code - 32));
    } else {
      return c;
    }
  } else {
    return c;
  }
}

export function keyword_variant__lto_1ba4622a(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__lto_self_97) => {
    return __k(((a, b) => {
      return (a + b);
    })(__lto_self_97, "_KW"));
  });
}

export function to_upper_string_loop__lto_1fab3ad0(s, i, acc) {
  const __lto_b_104 = String.len(s);
  const __match_102 = ((i < __lto_b_104) ? Ordering["less"] : ((__match_101) => {
    if (__match_101) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_104)));
  if (((__match_102[LUMO_TAG] === "less") ? false : ((__match_102[LUMO_TAG] === "equal") ? true : true))) {
    return acc;
  } else {
    return to_upper_string_loop__lto_1fab3ad0(s, ((__lto_self_105) => {
      return (__lto_self_105 + 1);
    })(i), ((__lto_self_109) => {
      return (__lto_self_109 + to_upper_char__lto_f0f5f7cb(String.char_at(s, i)));
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
      return String.char_code_at(__caps, name, 0, (code) => {
        const __match_143 = ((code < 65) ? Ordering["less"] : ((__match_142) => {
          if (__match_142) {
            return Ordering["equal"];
          } else {
            return Ordering["greater"];
          }
        })(((a, b) => {
          return (a === b);
        })(code, 65)));
        if (((__match_143[LUMO_TAG] === "less") ? false : ((__match_143[LUMO_TAG] === "equal") ? true : true))) {
          const __match_140 = ((code < 90) ? Ordering["less"] : ((__match_139) => {
            if (__match_139) {
              return Ordering["equal"];
            } else {
              return Ordering["greater"];
            }
          })(((a, b) => {
            return (a === b);
          })(code, 90)));
          if (((__match_140[LUMO_TAG] === "less") ? true : ((__match_140[LUMO_TAG] === "equal") ? true : false))) {
            return collect_tokens_from_alts__lto_9309ae26(__caps, rest, kws, syms, __k);
          } else {
            return collect_alt_token(__caps, name, rest, kws, syms, __k);
          }
        } else {
          return collect_alt_token(__caps, name, rest, kws, syms, __k);
        }
      });
    }
  });
}

export function string_lt_loop__lto_090deca7(a, b, i) {
  const __lto_b_248 = String.len(a);
  const __match_146 = ((i < __lto_b_248) ? Ordering["less"] : ((__match_145) => {
    if (__match_145) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_248)));
  if (((__match_146[LUMO_TAG] === "less") ? false : ((__match_146[LUMO_TAG] === "equal") ? true : true))) {
    let __match_163;
    let __match_162;
    const __lto_b_252 = String.len(b);
    if ((i < __lto_b_252)) {
      __match_162 = Ordering["less"];
    } else if ((i === __lto_b_252)) {
      __match_162 = Ordering["equal"];
    } else {
      __match_162 = Ordering["greater"];
    }
    if ((__match_162[LUMO_TAG] === "less")) {
      __match_163 = false;
    } else if ((__match_162[LUMO_TAG] === "equal")) {
      __match_163 = true;
    } else {
      __match_163 = true;
    }
    if (__match_163) {
      return false;
    } else {
      return true;
    }
  } else {
    let __match_151;
    let __match_150;
    const __lto_b_256 = String.len(b);
    if ((i < __lto_b_256)) {
      __match_150 = Ordering["less"];
    } else if ((i === __lto_b_256)) {
      __match_150 = Ordering["equal"];
    } else {
      __match_150 = Ordering["greater"];
    }
    if ((__match_150[LUMO_TAG] === "less")) {
      __match_151 = false;
    } else if ((__match_150[LUMO_TAG] === "equal")) {
      __match_151 = true;
    } else {
      __match_151 = true;
    }
    if (__match_151) {
      return false;
    } else {
      const ca = String.char_code_at(i, i);
      const cb = String.char_code_at(b, i);
      const __match_154 = ((ca < cb) ? Ordering["less"] : ((__match_153) => {
        if (__match_153) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(ca, cb)));
      if (((__match_154[LUMO_TAG] === "less") ? true : ((__match_154[LUMO_TAG] === "equal") ? false : false))) {
        return true;
      } else {
        let __match_159;
        let __match_158;
        if ((i < ca)) {
          __match_158 = Ordering["less"];
        } else if ((cb === ca)) {
          __match_158 = Ordering["equal"];
        } else {
          __match_158 = Ordering["greater"];
        }
        if ((__match_158[LUMO_TAG] === "less")) {
          __match_159 = true;
        } else if ((__match_158[LUMO_TAG] === "equal")) {
          __match_159 = false;
        } else {
          __match_159 = false;
        }
        if (__match_159) {
          return false;
        } else {
          return string_lt_loop__lto_090deca7(i, ca, ((__lto_self_265) => {
            return (__lto_self_265 + 1);
          })(i));
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
    const __match_169 = ((code < 65) ? Ordering["less"] : ((__match_168) => {
      if (__match_168) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(code, 65)));
    if ((((__match_169[LUMO_TAG] === "less") ? false : ((__match_169[LUMO_TAG] === "equal") ? true : true)) ? ((__match_173) => {
      if ((__match_173[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_173[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return false;
      }
    })(((__lto_self_273) => {
      const __lto_other_274 = 90;
      const __match_171 = (__lto_self_273 < __lto_other_274);
      if (__match_171) {
        return Ordering["less"];
      } else {
        const __match_172 = (__lto_self_273 === __lto_other_274);
        if (__match_172) {
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
    return emit_named_tokens__lto_1ba4622a(__caps, ((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n\n") + "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n") + "#[repr(u16)]\n") + "pub enum SyntaxKind {\n") + "    // Named tokens\n"), grammar.args[0], (s) => {
      return emit_keywords__lto_1ba4622a(__caps, ((s + "    // Trivia\n") + "    WHITESPACE,\n    NEWLINE,\n    UNKNOWN,\n"), keywords, (s) => {
        return emit_node_kinds__lto_1ba4622a(__caps, (emit_symbols__lto_1ba4622a(s, symbols) + "    // Nodes\n"), grammar.args[1], (s) => {
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
      return to_screaming_snake(__caps, tokens.args[0], (__lto_other_340) => {
        return emit_named_tokens__lto_1ba4622a(__caps, (((s + "    ") + __lto_other_340) + ",\n"), tokens.args[1], __k);
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
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_360) => {
        return emit_keywords_items__lto_1ba4622a(__caps, ((__lto_self_369) => {
          return (__lto_self_369 + (((("    " + __lto_other_360) + ", // '") + kw) + "'\n"));
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
    return emit_symbols_items__lto_1ba4622a(((syms.args[1][LUMO_TAG] === "nil") ? s : ((__match_181) => {
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
    return emit_symbols_items__lto_1ba4622a(((__lto_self_393) => {
      return (__lto_self_393 + line);
    })(s), syms.args[1]);
  }
}

export function emit_node_kinds__lto_1ba4622a(__caps, s, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const rest = rules.args[1];
      const __match_184 = rules.args[0];
      const name = __match_184.args[0];
      const __match_185 = __match_184.args[1];
      if ((__match_185[LUMO_TAG] === "sequence")) {
        const elems = __match_185.args[0];
        return to_screaming_snake(__caps, name, (__lto_other_404) => {
          return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_413) => {
            return (__lto_self_413 + (((("    " + __lto_other_404) + ", // ") + name) + "\n"));
          })(s), rest, __k);
        });
      } else if (is_token_only_alternatives__lto_9309ae26(__match_185.args[0])) {
        return to_screaming_snake(__caps, name, (__lto_other_424) => {
          return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_433) => {
            return (__lto_self_433 + (((("    " + __lto_other_424) + ", // ") + name) + " (token wrapper)\n"));
          })(s), rest, __k);
        });
      } else {
        return emit_node_kinds__lto_1ba4622a(__caps, s, rest, __k);
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
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_460) => {
        return emit_keyword_arms__lto_1ba4622a(__caps, ((__lto_self_473) => {
          return (__lto_self_473 + (((("            \"" + kw) + "\" => Some(Self::") + __lto_other_460) + "),\n"));
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
    return emit_symbol_arms__lto_1ba4622a(((__lto_self_513) => {
      return (__lto_self_513 + line);
    })(s), syms.args[1]);
  }
}

export function generate_ast__lto_1ba4622a(__caps, grammar, __k) {
  return __thunk(() => {
    return emit_ast_rules(__caps, ((((((("// Auto-generated by langue. Do not edit.\n" + "// Regenerate: scripts/gen_langue.sh\n\n") + "use super::SyntaxKind;\n") + "use super::{SyntaxNode, SyntaxElement, LosslessToken};\n\n") + "pub trait AstNode<'a>: Sized {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self>;\n") + "    fn syntax(&self) -> &'a SyntaxNode;\n") + "}\n\n"), grammar.args[0], grammar.args[1], __k);
  });
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
    return emit_enum_variants__lto_1ba4622a(((__lto_self_849) => {
      return (__lto_self_849 + "<'a>),\n");
    })(((__lto_self_851) => {
      return (__lto_self_851 + name);
    })(((__lto_self_853) => {
      return (__lto_self_853 + "(");
    })(((__lto_self_855) => {
      return (__lto_self_855 + name);
    })(((__lto_self_857) => {
      return (__lto_self_857 + "    ");
    })(s))))), alts.args[1]);
  }
}

export function emit_enum_cast_chain__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const name = alts.args[0].args[0];
    return emit_enum_cast_chain__lto_1ba4622a(((__lto_self_885) => {
      return (__lto_self_885 + (((("            .or_else(|| " + name) + "::cast(node).map(Self::") + name) + "))\n"));
    })(s), alts.args[1]);
  }
}

export function emit_enum_syntax_arms__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else {
    return emit_enum_syntax_arms__lto_1ba4622a(((__lto_self_897) => {
      return (__lto_self_897 + (("            Self::" + alts.args[0].args[0]) + "(n) => n.syntax(),\n"));
    })(s), alts.args[1]);
  }
}

export function emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return __k((((((((((((((s + "pub struct ") + name) + "<'a>(pub(crate) &'a SyntaxNode);\n\n") + "impl<'a> AstNode<'a> for ") + name) + "<'a> {\n") + "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n") + "        (node.kind == SyntaxKind::") + kind) + ").then(|| Self(node))\n") + "    }\n") + "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n") + "}\n\n"));
  });
}

export function run__lto_3829b133(__caps, __k) {
  return __thunk(() => {
    const __lto_a_955 = (__argv_length_raw() - 1);
    const __match_206 = ((__lto_a_955 < 2) ? Ordering["less"] : ((__match_205) => {
      if (__match_205) {
        return Ordering["equal"];
      } else {
        return Ordering["greater"];
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_a_955, 2)));
    if (((__match_206[LUMO_TAG] === "less") ? true : ((__match_206[LUMO_TAG] === "equal") ? false : false))) {
      const __lto__err_958 = __console_error("Usage: langue <input.langue> [output_dir]");
      return __k(__exit_process(1));
    } else {
      const file = __argv_at_raw(((__lto___lto_self_1217_1230) => {
        return (__lto___lto_self_1217_1230 + 1);
      })(1));
      return parse_grammar(__caps, readFileSync(file, "utf8"), (__cps_v_32) => {
        if ((__cps_v_32[LUMO_TAG] === "ok")) {
          return resolve_grammar(__caps, __cps_v_32.args[0], (grammar) => {
            const tokens = grammar.args[0];
            const count = list_length_rules__lto_92991de6(grammar.args[1]);
            return generate_syntax_kind__lto_1ba4622a(__caps, grammar, (syntax_kind_code) => {
              return generate_ast__lto_1ba4622a(__caps, grammar, (ast_code) => {
                return __k(run_generate__lto_35421161(file, count, syntax_kind_code, ast_code));
              });
            });
          });
        } else {
          return Number.to_string(__caps, __cps_v_32.args[1], (__lto_other_968) => {
            const __lto__err_962 = __console_error(((("Parse error at position " + __lto_other_968) + ": ") + __cps_v_32.args[0]));
            return __k(__exit_process(1));
          });
        }
      });
    }
  });
}

export function run_generate__lto_35421161(file, count, syntax_kind_code, ast_code) {
  const __lto_a_977 = (__argv_length_raw() - 1);
  const __match_209 = ((__lto_a_977 < 3) ? Ordering["less"] : ((__match_208) => {
    if (__match_208) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_977, 3)));
  if (((__match_209[LUMO_TAG] === "less") ? true : ((__match_209[LUMO_TAG] === "equal") ? false : false))) {
    return write_output__lto_155dcaa4(".", file, count, syntax_kind_code, ast_code);
  } else {
    return write_output__lto_155dcaa4(__argv_at_raw(((__lto___lto_self_1217_1239) => {
      return (__lto___lto_self_1217_1239 + 1);
    })(2)), file, count, syntax_kind_code, ast_code);
  }
}

export function write_output__lto_155dcaa4(out_dir, file, count, syntax_kind_code, ast_code) {
  const sk_path = (out_dir + "/syntax_kind.rs");
  const ast_path = (out_dir + "/ast.rs");
  const w1 = writeFileSync(sk_path, syntax_kind_code, "utf8");
  const w2 = writeFileSync(ast_path, ast_code, "utf8");
  const p1 = globalThis.console.log(((("Parsed " + Number.to_string(count)) + " rules from ") + file));
  const p2 = globalThis.console.log(("Wrote " + sk_path));
  return globalThis.console.log(("Wrote " + ast_path));
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
  const __match_218 = ((code < 97) ? Ordering["less"] : ((__match_217) => {
    if (__match_217) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(code, 97)));
  if (((__match_218[LUMO_TAG] === "less") ? false : ((__match_218[LUMO_TAG] === "equal") ? true : true))) {
    let __match_229;
    if ((code < 122)) {
      __match_229 = Ordering["less"];
    } else if ((code === 122)) {
      __match_229 = Ordering["equal"];
    } else {
      __match_229 = Ordering["greater"];
    }
    if ((__match_229[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_229[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return false;
    }
  } else {
    let __match_223;
    let __match_222;
    if ((code < 65)) {
      __match_222 = Ordering["less"];
    } else if ((code === 65)) {
      __match_222 = Ordering["equal"];
    } else {
      __match_222 = Ordering["greater"];
    }
    if ((__match_222[LUMO_TAG] === "less")) {
      __match_223 = false;
    } else if ((__match_222[LUMO_TAG] === "equal")) {
      __match_223 = true;
    } else {
      __match_223 = true;
    }
    if (__match_223) {
      let __match_226;
      if ((code < 90)) {
        __match_226 = Ordering["less"];
      } else if ((code === 90)) {
        __match_226 = Ordering["equal"];
      } else {
        __match_226 = Ordering["greater"];
      }
      if ((__match_226[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_226[LUMO_TAG] === "equal")) {
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

export function state_eof__lto_9309ae26(st) {
  const __lto_a_1057 = st.args[1];
  const __lto_b_1058 = String.len(st.args[0]);
  const __match_235 = ((__lto_a_1057 < __lto_b_1058) ? Ordering["less"] : ((__match_234) => {
    if (__match_234) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_1057, __lto_b_1058)));
  if ((__match_235[LUMO_TAG] === "less")) {
    return false;
  } else if ((__match_235[LUMO_TAG] === "equal")) {
    return true;
  } else {
    return true;
  }
}

export function state_peek__lto_9309ae26(st) {
  const src = st.args[0];
  const pos = st.args[1];
  const __lto_b_1062 = String.len(src);
  const __match_239 = ((pos < __lto_b_1062) ? Ordering["less"] : ((__match_238) => {
    if (__match_238) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(pos, __lto_b_1062)));
  if (((__match_239[LUMO_TAG] === "less") ? true : ((__match_239[LUMO_TAG] === "equal") ? false : false))) {
    return String.char_at(src, pos);
  } else {
    return "";
  }
}

export function state_advance__lto_92991de6(st, n) {
  return ParseState["mk"](st.args[0], ((__lto_self_1063) => {
    return (__lto_self_1063 + n);
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
      const __lto_b_1078 = String.len(state_src(st));
      const __match_247 = ((next_pos < __lto_b_1078) ? Ordering["less"] : ((__match_246) => {
        if (__match_246) {
          return Ordering["equal"];
        } else {
          return Ordering["greater"];
        }
      })(((a, b) => {
        return (a === b);
      })(next_pos, __lto_b_1078)));
      if (((__match_247[LUMO_TAG] === "less") ? true : ((__match_247[LUMO_TAG] === "equal") ? false : false))) {
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
      return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_34) => {
        if (__cps_v_34) {
          const start = state_pos(st2);
          return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (end_st) => {
            return String.slice(__caps, state_src(st2), start, state_pos(end_st), (__cps_v_33) => {
              return __k(ParseResult["ok"](__cps_v_33, end_st));
            });
          });
        } else {
          return __k(ParseResult["err"](((__lto_self_1087) => {
            return (__lto_self_1087 + "'");
          })(((__lto_self_1089) => {
            return (__lto_self_1089 + state_peek__lto_9309ae26(st2));
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
  const __lto_a_1101 = (String.len(src) - pos);
  const __match_256 = ((__lto_a_1101 < len) ? Ordering["less"] : ((__match_255) => {
    if (__match_255) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_a_1101, len)));
  if (((__match_256[LUMO_TAG] === "less") ? false : ((__match_256[LUMO_TAG] === "equal") ? true : true))) {
    const slice = String.slice(src, pos, ((__lto_self_1103) => {
      return (__lto_self_1103 + len);
    })(pos));
    if ((slice === expected)) {
      return ParseResult["ok"](expected, state_advance__lto_92991de6(st2, len));
    } else {
      return ParseResult["err"](((__lto_self_1111) => {
        return (__lto_self_1111 + "'");
      })(((__lto_self_1113) => {
        return (__lto_self_1113 + slice);
      })(((__lto_self_1115) => {
        return (__lto_self_1115 + "', got '");
      })(((__lto_self_1117) => {
        return (__lto_self_1117 + expected);
      })("expected '")))), pos);
    }
  } else {
    return ParseResult["err"](((__lto_self_1127) => {
      return (__lto_self_1127 + "'");
    })(((__lto_self_1129) => {
      return (__lto_self_1129 + expected);
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

export function peek_is_rule_start__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_35) => {
      if (__cps_v_35) {
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
  const __lto_b_1154 = String.len(s);
  const __match_265 = ((i < __lto_b_1154) ? Ordering["less"] : ((__match_264) => {
    if (__match_264) {
      return Ordering["equal"];
    } else {
      return Ordering["greater"];
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_b_1154)));
  if (((__match_265[LUMO_TAG] === "less") ? false : ((__match_265[LUMO_TAG] === "equal") ? true : true))) {
    return false;
  } else if (is_alpha__lto_9309ae26(String.char_at(s, i))) {
    return true;
  } else {
    return has_alpha__lto_090deca7(s, ((__lto_self_1155) => {
      return (__lto_self_1155 + 1);
    })(i));
  }
}

export function parse_grammar_items__lto_3890158f(__caps, st, tokens, rules, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    if (state_eof__lto_9309ae26(st2)) {
      return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_rule(rules)), st2));
    } else if ((state_peek__lto_9309ae26(st2) === "@")) {
      return parse_token_def(__caps, st2, (__cps_v_37) => {
        if ((__cps_v_37[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_37.args[1], list_concat_string(__cps_v_37.args[0], tokens), rules, __k);
        } else {
          return __k(ParseResult["err"](__cps_v_37.args[0], __cps_v_37.args[1]));
        }
      });
    } else {
      return parse_rule(__caps, st2, (__cps_v_36) => {
        if ((__cps_v_36[LUMO_TAG] === "ok")) {
          return parse_grammar_items__lto_3890158f(__caps, __cps_v_36.args[1], tokens, List["cons"](__cps_v_36.args[0], rules), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_36.args[0], __cps_v_36.args[1]));
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
      return peek_is_rule_start__lto_3890158f(__caps, st2, (__cps_v_40) => {
        if (__cps_v_40) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((state_peek__lto_9309ae26(st2) === "@")) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else {
          return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_39) => {
            if (__cps_v_39) {
              return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_38) => {
                if ((__cps_v_38[LUMO_TAG] === "ok")) {
                  return parse_token_names__lto_3890158f(__caps, __cps_v_38.args[1], List["cons"](__cps_v_38.args[0], acc), __k);
                } else {
                  return __k(ParseResult["err"](__cps_v_38.args[0], __cps_v_38.args[1]));
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
    return peek_char(__caps, st2, (__lto_self_1167) => {
      if ((__lto_self_1167 === "|")) {
        return parse_alternatives(__caps, st2, __k);
      } else {
        return parse_sequence(__caps, st2, __k);
      }
    });
  });
}

export function parse_alt_items__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_char(__caps, st2, (__lto_self_1171) => {
      if ((__lto_self_1171 === "|")) {
        const st3 = state_advance__lto_92991de6(skip_ws__lto_1bb67705(st2), 1);
        const st4 = skip_ws__lto_1bb67705(st3);
        if ((state_peek__lto_9309ae26(st4) === "'")) {
          const __match_281 = parse_quoted__lto_38e07bea(st4);
          if ((__match_281[LUMO_TAG] === "ok")) {
            return parse_alt_items__lto_3890158f(__caps, __match_281.args[1], List["cons"](Alternative["mk"](__match_281.args[0]), acc), __k);
          } else {
            return __k(ParseResult["err"](__match_281.args[0], __match_281.args[1]));
          }
        } else {
          return parse_ident__lto_1ba4622a(__caps, st3, (__cps_v_42) => {
            if ((__cps_v_42[LUMO_TAG] === "ok")) {
              return parse_alt_items__lto_3890158f(__caps, __cps_v_42.args[1], List["cons"](Alternative["mk"](__cps_v_42.args[0]), acc), __k);
            } else {
              return __k(ParseResult["err"](__cps_v_42.args[0], __cps_v_42.args[1]));
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
      return peek_is_rule_start__lto_3890158f(__caps, st, (__cps_v_44) => {
        if (__cps_v_44) {
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
      const __match_295 = parse_quoted__lto_38e07bea(st2);
      if ((__match_295[LUMO_TAG] === "ok")) {
        return classify_literal(__caps, __match_295.args[0], (__cps_v_49) => {
          return __k(ParseResult["ok"](Element["token"](__cps_v_49), __match_295.args[1]));
        });
      } else {
        return __k(ParseResult["err"](__match_295.args[0], __match_295.args[1]));
      }
    } else if ((state_peek__lto_9309ae26(st2) === "(")) {
      return parse_group_elements__lto_3890158f(__caps, state_advance__lto_92991de6(st2, 1), List["nil"], (__cps_v_47) => {
        if ((__cps_v_47[LUMO_TAG] === "ok")) {
          const __match_294 = expect__lto_f3280589(__cps_v_47.args[1], ")");
          if ((__match_294[LUMO_TAG] === "ok")) {
            return __k(ParseResult["ok"](Element["group"](__cps_v_47.args[0]), __match_294.args[1]));
          } else {
            return __k(ParseResult["err"](__match_294.args[0], __match_294.args[1]));
          }
        } else {
          return __k(ParseResult["err"](__cps_v_47.args[0], __cps_v_47.args[1]));
        }
      });
    } else {
      return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_46) => {
        if ((__cps_v_46[LUMO_TAG] === "err")) {
          return __k(ParseResult["err"](__cps_v_46.args[0], __cps_v_46.args[1]));
        } else {
          const name = __cps_v_46.args[0];
          const st3 = __cps_v_46.args[1];
          if ((state_peek__lto_9309ae26(st3) === ":")) {
            return parse_element(__caps, state_advance__lto_92991de6(st3, 1), (__cps_v_45) => {
              if ((__cps_v_45[LUMO_TAG] === "ok")) {
                return __k(ParseResult["ok"](Element["labeled"](name, __cps_v_45.args[0]), __cps_v_45.args[1]));
              } else {
                return __k(ParseResult["err"](__cps_v_45.args[0], __cps_v_45.args[1]));
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
      return parse_element(__caps, st2, (__cps_v_50) => {
        if ((__cps_v_50[LUMO_TAG] === "ok")) {
          return parse_group_elements__lto_3890158f(__caps, __cps_v_50.args[1], List["cons"](__cps_v_50.args[0], acc), __k);
        } else {
          return __k(ParseResult["err"](__cps_v_50.args[0], __cps_v_50.args[1]));
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

main();
