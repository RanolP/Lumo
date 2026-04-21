const LUMO_TAG = Symbol.for("Lumo/tag");
const __lumo_match_error = (value) => { throw new Error("non-exhaustive match: " + JSON.stringify(value)); };
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
    if ((grammar[LUMO_TAG] === "mk")) {
      const token_defs = grammar.args[0];
      const rules = grammar.args[1];
      return collect_tokens_from_rules(__caps, rules, List["nil"], List["nil"], (pair) => {
        if ((pair[LUMO_TAG] === "mk")) {
          const kws = pair.args[0];
          const syms = pair.args[1];
          return dedupe_strings(__caps, kws, (__cps_v_3) => {
            return sort_strings(__caps, __cps_v_3, (__cps_v_0) => {
              return dedupe_strings(__caps, syms, (__cps_v_2) => {
                return sort_strings(__caps, __cps_v_2, (__cps_v_1) => {
                  return __k(CollectedTokens["mk"](__cps_v_0, __cps_v_1));
                });
              });
            });
          });
        } else {
          return __lumo_match_error(pair);
        }
      });
    } else {
      return __lumo_match_error(grammar);
    }
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
    } else if ((rules[LUMO_TAG] === "cons")) {
      const rule = rules.args[0];
      const rest = rules.args[1];
      if ((rule[LUMO_TAG] === "mk")) {
        const name = rule.args[0];
        const body = rule.args[1];
        return collect_tokens_from_body(__caps, body, kws, syms, (pair) => {
          if ((pair[LUMO_TAG] === "mk")) {
            const kws2 = pair.args[0];
            const syms2 = pair.args[1];
            return collect_tokens_from_rules(__caps, rest, kws2, syms2, __k);
          } else {
            return __lumo_match_error(pair);
          }
        });
      } else {
        return __lumo_match_error(rule);
      }
    } else {
      return __lumo_match_error(rules);
    }
  });
}

export function collect_tokens_from_body(__caps, body, kws, syms, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      const elems = body.args[0];
      return __k(collect_tokens_from_elements(elems, kws, syms));
    } else if ((body[LUMO_TAG] === "alternatives")) {
      const alts = body.args[0];
      return collect_tokens_from_alts__lto_9309ae26(__caps, alts, kws, syms, __k);
    } else {
      return __lumo_match_error(body);
    }
  });
}

export function collect_alt_token(__caps, name, rest, kws, syms, __k) {
  return __thunk(() => {
    const __match_6 = has_alpha__lto_090deca7(name, 0);
    if ((__match_6 === true)) {
      return collect_tokens_from_alts__lto_9309ae26(__caps, rest, List["cons"](name, kws), syms, __k);
    } else if ((__match_6 === false)) {
      return collect_tokens_from_alts__lto_9309ae26(__caps, rest, kws, List["cons"](name, syms), __k);
    } else {
      return __lumo_match_error(__match_6);
    }
  });
}

export function collect_tokens_from_elements(elems, kws, syms) {
  if ((elems[LUMO_TAG] === "nil")) {
    return StringPair["mk"](kws, syms);
  } else if ((elems[LUMO_TAG] === "cons")) {
    const elem = elems.args[0];
    const rest = elems.args[1];
    const pair = collect_tokens_from_element(elem, kws, syms);
    if ((pair[LUMO_TAG] === "mk")) {
      const kws2 = pair.args[0];
      const syms2 = pair.args[1];
      return collect_tokens_from_elements(rest, kws2, syms2);
    } else {
      return __lumo_match_error(pair);
    }
  } else {
    return __lumo_match_error(elems);
  }
}

export function collect_tokens_from_element(elem, kws, syms) {
  if ((elem[LUMO_TAG] === "token")) {
    const t = elem.args[0];
    if ((t[LUMO_TAG] === "keyword")) {
      const k = t.args[0];
      return StringPair["mk"](List["cons"](k, kws), syms);
    } else if ((t[LUMO_TAG] === "symbol")) {
      const s = t.args[0];
      return StringPair["mk"](kws, List["cons"](s, syms));
    } else {
      return ((t[LUMO_TAG] === "named") ? ((n) => {
        return StringPair["mk"](kws, syms);
      })(t.args[0]) : __lumo_match_error(t));
    }
  } else if ((elem[LUMO_TAG] === "node")) {
    const ref = elem.args[0];
    return StringPair["mk"](kws, syms);
  } else {
    return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
      const inner = elem.args[1];
      return collect_tokens_from_element(inner, kws, syms);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
      return collect_tokens_from_element(inner, kws, syms);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
      return collect_tokens_from_element(inner, kws, syms);
    })(elem.args[0]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
      return collect_tokens_from_elements(elems, kws, syms);
    })(elem.args[0]) : __lumo_match_error(elem)))));
  }
}

export function dedupe_strings(__caps, xs, __k) {
  return dedupe_strings_acc(__caps, xs, List["nil"], __k);
}

export function dedupe_strings_acc(__caps, xs, acc, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(acc);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const x = xs.args[0];
      const rest = xs.args[1];
      const __match_12 = list_contains_string__lto_3890158f(acc, x);
      if ((__match_12 === true)) {
        return dedupe_strings_acc(__caps, rest, acc, __k);
      } else if ((__match_12 === false)) {
        return dedupe_strings_acc(__caps, rest, List["cons"](x, acc), __k);
      } else {
        return __lumo_match_error(__match_12);
      }
    } else {
      return __lumo_match_error(xs);
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
    } else if ((xs[LUMO_TAG] === "cons")) {
      const x = xs.args[0];
      const rest = xs.args[1];
      return insert_sorted(__caps, x, sorted, (__cps_v_4) => {
        return sort_strings_acc(__caps, rest, __cps_v_4, __k);
      });
    } else {
      return __lumo_match_error(xs);
    }
  });
}

export function insert_sorted(__caps, s, xs, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(List["cons"](s, xs));
    } else if ((xs[LUMO_TAG] === "cons")) {
      const x = xs.args[0];
      const rest = xs.args[1];
      return string_lt(__caps, s, x, (__cps_v_6) => {
        if ((__cps_v_6 === true)) {
          return __k(List["cons"](s, xs));
        } else if ((__cps_v_6 === false)) {
          return insert_sorted(__caps, s, rest, (__cps_v_5) => {
            return __k(List["cons"](x, __cps_v_5));
          });
        } else {
          return __lumo_match_error(__cps_v_6);
        }
      });
    } else {
      return __lumo_match_error(xs);
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
    } else if ((rules[LUMO_TAG] === "cons")) {
      const rule = rules.args[0];
      const rest = rules.args[1];
      if ((rule[LUMO_TAG] === "mk")) {
        const name = rule.args[0];
        const body = rule.args[1];
        const __k_14 = (s2) => {
          return emit_ast_rules(__caps, s2, token_defs, rest, __k);
        };
        if ((body[LUMO_TAG] === "sequence")) {
          const elems = body.args[0];
          return emit_struct_node__lto_1ba4622a(__caps, s, name, elems, token_defs, __k_14);
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          const __match_19 = is_token_only_alternatives__lto_9309ae26(alts);
          if ((__match_19 === true)) {
            return emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k_14);
          } else if ((__match_19 === false)) {
            return __k_14(emit_enum_node__lto_1ba4622a(s, name, alts));
          } else {
            return __lumo_match_error(__match_19);
          }
        } else {
          return __lumo_match_error(body);
        }
      } else {
        return __lumo_match_error(rule);
      }
    } else {
      return __lumo_match_error(rules);
    }
  });
}

export function has_labeled_elements(elems) {
  if ((elems[LUMO_TAG] === "nil")) {
    return false;
  } else if ((elems[LUMO_TAG] === "cons")) {
    const elem = elems.args[0];
    const rest = elems.args[1];
    if ((elem[LUMO_TAG] === "labeled")) {
      return true;
    } else {
      return has_labeled_elements(rest);
    }
  } else {
    return __lumo_match_error(elems);
  }
}

export function emit_accessors_for_elements(__caps, s, elems, token_defs, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((elems[LUMO_TAG] === "cons")) {
      const elem = elems.args[0];
      const rest = elems.args[1];
      if ((elem[LUMO_TAG] === "labeled")) {
        const label = elem.args[0];
        const inner = elem.args[1];
        return emit_single_accessor(__caps, s, label, inner, token_defs, (s2) => {
          return emit_accessors_for_elements(__caps, s2, rest, token_defs, __k);
        });
      } else {
        return emit_accessors_for_elements(__caps, s, rest, token_defs, __k);
      }
    } else {
      return __lumo_match_error(elems);
    }
  });
}

export function emit_single_accessor(__caps, s, label, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const t = elem.args[0];
      return emit_token_accessor__lto_1ba4622a(__caps, s, label, t, false, __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        const __match_26 = list_contains_string__lto_3890158f(token_defs, name);
        if ((__match_26 === true)) {
          return emit_token_accessor__lto_1ba4622a(__caps, s, label, TokenRef["named"](name), false, __k);
        } else if ((__match_26 === false)) {
          return __k(emit_node_accessor__lto_1ba4622a(s, label, name, false));
        } else {
          return __lumo_match_error(__match_26);
        }
      } else {
        return __lumo_match_error(n);
      }
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_single_accessor(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "labeled") ? ((inner) => {
        return emit_single_accessor(__caps, s, label, inner, token_defs, __k);
      })(elem.args[1]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
        return __k(s);
      })(elem.args[0]) : __lumo_match_error(elem)))));
    }
  });
}

export function emit_single_accessor_repeated(__caps, s, label, elem, token_defs, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const t = elem.args[0];
      return emit_token_accessor__lto_1ba4622a(__caps, s, label, t, true, __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        const __match_29 = list_contains_string__lto_3890158f(token_defs, name);
        if ((__match_29 === true)) {
          return emit_token_accessor__lto_1ba4622a(__caps, s, label, TokenRef["named"](name), true, __k);
        } else if ((__match_29 === false)) {
          return __k(emit_node_accessor__lto_1ba4622a(s, label, name, true));
        } else {
          return __lumo_match_error(__match_29);
        }
      } else {
        return __lumo_match_error(n);
      }
    } else {
      return ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[0]) : ((elem[LUMO_TAG] === "labeled") ? ((inner) => {
        return emit_single_accessor_repeated(__caps, s, label, inner, token_defs, __k);
      })(elem.args[1]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
        return __k(s);
      })(elem.args[0]) : __lumo_match_error(elem)))));
    }
  });
}

export function token_kind_from_ref(__caps, t, __k) {
  return __thunk(() => {
    if ((t[LUMO_TAG] === "named")) {
      const name = t.args[0];
      return to_screaming_snake(__caps, name, __k);
    } else if ((t[LUMO_TAG] === "keyword")) {
      const kw = t.args[0];
      return keyword_variant__lto_1ba4622a(__caps, kw, __k);
    } else {
      return ((t[LUMO_TAG] === "symbol") ? ((sym) => {
        return __k(symbol_variant__lto_8227044e(sym));
      })(t.args[0]) : __lumo_match_error(t));
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
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    return src;
  } else {
    return __lumo_match_error(st);
  }
}

export function state_pos(st) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    return pos;
  } else {
    return __lumo_match_error(st);
  }
}

export function scan_ident_rest(__caps, st, __k) {
  return __thunk(() => {
    const __match_33 = state_eof__lto_9309ae26(st);
    if ((__match_33 === true)) {
      return __k(st);
    } else if ((__match_33 === false)) {
      const __match_34 = is_ident_continue__lto_3890158f(state_peek__lto_9309ae26(st));
      if ((__match_34 === true)) {
        return scan_ident_rest(__caps, state_advance__lto_92991de6(st, 1), __k);
      } else if ((__match_34 === false)) {
        return __k(st);
      } else {
        return __lumo_match_error(__match_34);
      }
    } else {
      return __lumo_match_error(__match_33);
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
    const __match_35 = has_alpha__lto_090deca7(text, 0);
    if ((__match_35 === true)) {
      return __k(TokenRef["keyword"](text));
    } else if ((__match_35 === false)) {
      return __k(TokenRef["symbol"](text));
    } else {
      return __lumo_match_error(__match_35);
    }
  });
}

export function parse_grammar(__caps, src, __k) {
  return __thunk(() => {
    const st = ParseState["mk"](src, 0);
    return parse_grammar_items__lto_3890158f(__caps, st, List["nil"], List["nil"], __k);
  });
}

export function parse_token_def(__caps, st, __k) {
  return __thunk(() => {
    const __match_36 = expect__lto_f3280589(st, "@token");
    if ((__match_36[LUMO_TAG] === "err")) {
      const msg = __match_36.args[0];
      const pos = __match_36.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__match_36[LUMO_TAG] === "ok")) {
      const st2 = __match_36.args[1];
      return parse_token_names__lto_3890158f(__caps, st2, List["nil"], __k);
    } else {
      return __lumo_match_error(__match_36);
    }
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident__lto_1ba4622a(__caps, st, (__cps_v_8) => {
    if ((__cps_v_8[LUMO_TAG] === "err")) {
      const msg = __cps_v_8.args[0];
      const pos = __cps_v_8.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_8[LUMO_TAG] === "ok")) {
      const name = __cps_v_8.args[0];
      const st2 = __cps_v_8.args[1];
      const __match_38 = expect__lto_f3280589(st2, "=");
      if ((__match_38[LUMO_TAG] === "err")) {
        const msg = __match_38.args[0];
        const pos = __match_38.args[1];
        return __k(ParseResult["err"](msg, pos));
      } else if ((__match_38[LUMO_TAG] === "ok")) {
        const st3 = __match_38.args[1];
        return parse_rule_body__lto_3890158f(__caps, st3, name, (__cps_v_7) => {
          if ((__cps_v_7[LUMO_TAG] === "err")) {
            const msg = __cps_v_7.args[0];
            const pos = __cps_v_7.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else if ((__cps_v_7[LUMO_TAG] === "ok")) {
            const body = __cps_v_7.args[0];
            const st4 = __cps_v_7.args[1];
            return __k(ParseResult["ok"](Rule["mk"](name, body), st4));
          } else {
            return __lumo_match_error(__cps_v_7);
          }
        });
      } else {
        return __lumo_match_error(__match_38);
      }
    } else {
      return __lumo_match_error(__cps_v_8);
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
    const __match_40 = state_eof__lto_9309ae26(st2);
    if ((__match_40 === true)) {
      return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
    } else if ((__match_40 === false)) {
      return is_seq_terminator__lto_3890158f(__caps, st2, (__cps_v_10) => {
        if ((__cps_v_10 === true)) {
          return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
        } else if ((__cps_v_10 === false)) {
          return parse_element(__caps, st2, (__cps_v_9) => {
            if ((__cps_v_9[LUMO_TAG] === "ok")) {
              const elem = __cps_v_9.args[0];
              const st3 = __cps_v_9.args[1];
              return parse_seq_elements(__caps, st3, List["cons"](elem, acc), __k);
            } else if ((__cps_v_9[LUMO_TAG] === "err")) {
              const msg = __cps_v_9.args[0];
              const pos = __cps_v_9.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_9);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_10);
        }
      });
    } else {
      return __lumo_match_error(__match_40);
    }
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom__lto_3890158f(__caps, st, (__cps_v_11) => {
    if ((__cps_v_11[LUMO_TAG] === "err")) {
      const msg = __cps_v_11.args[0];
      const pos = __cps_v_11.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_11[LUMO_TAG] === "ok")) {
      const elem = __cps_v_11.args[0];
      const st2 = __cps_v_11.args[1];
      return __k(apply_postfix_elem__lto_3890158f(elem, st2));
    } else {
      return __lumo_match_error(__cps_v_11);
    }
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    if ((g[LUMO_TAG] === "mk")) {
      const token_defs = g.args[0];
      const rules = g.args[1];
      return resolve_rules(__caps, token_defs, rules, (__cps_v_12) => {
        return __k(Grammar["mk"](token_defs, __cps_v_12));
      });
    } else {
      return __lumo_match_error(g);
    }
  });
}

export function resolve_rules(__caps, token_defs, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else if ((rules[LUMO_TAG] === "cons")) {
      const rule = rules.args[0];
      const rest = rules.args[1];
      if ((rule[LUMO_TAG] === "mk")) {
        const name = rule.args[0];
        const body = rule.args[1];
        return resolve_body(__caps, token_defs, body, (resolved_body) => {
          return resolve_rules(__caps, token_defs, rest, (__cps_v_13) => {
            return __k(List["cons"](Rule["mk"](name, resolved_body), __cps_v_13));
          });
        });
      } else {
        return __lumo_match_error(rule);
      }
    } else {
      return __lumo_match_error(rules);
    }
  });
}

export function resolve_body(__caps, token_defs, body, __k) {
  return __thunk(() => {
    if ((body[LUMO_TAG] === "sequence")) {
      const elems = body.args[0];
      return resolve_elements(__caps, token_defs, elems, (__cps_v_14) => {
        return __k(RuleBody["sequence"](__cps_v_14));
      });
    } else if ((body[LUMO_TAG] === "alternatives")) {
      const alts = body.args[0];
      return __k(body);
    } else {
      return __lumo_match_error(body);
    }
  });
}

export function resolve_elements(__caps, token_defs, elems, __k) {
  return __thunk(() => {
    if ((elems[LUMO_TAG] === "nil")) {
      return __k(List["nil"]);
    } else if ((elems[LUMO_TAG] === "cons")) {
      const elem = elems.args[0];
      const rest = elems.args[1];
      return resolve_element(__caps, token_defs, elem, (__cps_v_15) => {
        return resolve_elements(__caps, token_defs, rest, (__cps_v_16) => {
          return __k(List["cons"](__cps_v_15, __cps_v_16));
        });
      });
    } else {
      return __lumo_match_error(elems);
    }
  });
}

export function resolve_element(__caps, token_defs, elem, __k) {
  return __thunk(() => {
    if ((elem[LUMO_TAG] === "token")) {
      const t = elem.args[0];
      return __k(elem);
    } else if ((elem[LUMO_TAG] === "node")) {
      const ref = elem.args[0];
      if ((ref[LUMO_TAG] === "mk")) {
        const name = ref.args[0];
        const __match_51 = list_contains_string__lto_3890158f(token_defs, name);
        if ((__match_51 === true)) {
          return __k(Element["token"](TokenRef["named"](name)));
        } else if ((__match_51 === false)) {
          return __k(elem);
        } else {
          return __lumo_match_error(__match_51);
        }
      } else {
        return __lumo_match_error(ref);
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        const inner = elem.args[1];
        return resolve_element(__caps, token_defs, inner, (__cps_v_20) => {
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
      })(elem.args[0]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_17) => {
          return __k(Element["group"](__cps_v_17));
        });
      })(elem.args[0]) : __lumo_match_error(elem)))));
    }
  });
}

export function list_reverse_string(xs) {
  return list_reverse_string_acc(xs, List["nil"]);
}

export function list_reverse_string_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    return list_reverse_string_acc(rest, List["cons"](x, acc));
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_reverse_rule(xs) {
  return list_reverse_rule_acc(xs, List["nil"]);
}

export function list_reverse_rule_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    return list_reverse_rule_acc(rest, List["cons"](x, acc));
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_reverse_alt(xs) {
  return list_reverse_alt_acc(xs, List["nil"]);
}

export function list_reverse_alt_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    return list_reverse_alt_acc(rest, List["cons"](x, acc));
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_reverse_elem(xs) {
  return list_reverse_elem_acc(xs, List["nil"]);
}

export function list_reverse_elem_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    return list_reverse_elem_acc(rest, List["cons"](x, acc));
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_concat_string(xs, ys) {
  if ((xs[LUMO_TAG] === "nil")) {
    return ys;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    return List["cons"](x, list_concat_string(rest, ys));
  } else {
    return __lumo_match_error(xs);
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
        if ((__match_57 === true)) {
          return false;
        } else if ((__match_57 === false)) {
          return true;
        } else {
          return __lumo_match_error(__match_57);
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
        if ((__match_58 === true)) {
          return Ordering["less"];
        } else if ((__match_58 === false)) {
          const __match_59 = (a === b);
          if ((__match_59 === true)) {
            return Ordering["equal"];
          } else if ((__match_59 === false)) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_59);
          }
        } else {
          return __lumo_match_error(__match_58);
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
  const __lto_other_1 = String.len(name);
  const __match_60 = (i < __lto_other_1);
  const __match_62 = ((__match_60 === true) ? Ordering["less"] : ((__match_60 === false) ? ((__match_61) => {
    if ((__match_61 === true)) {
      return Ordering["equal"];
    } else if ((__match_61 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_61);
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_other_1)) : __lumo_match_error(__match_60)));
  const __match_63 = ((__match_62[LUMO_TAG] === "less") ? false : ((__match_62[LUMO_TAG] === "equal") ? true : ((__match_62[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_62))));
  if ((__match_63 === true)) {
    return acc;
  } else if ((__match_63 === false)) {
    const c = String.char_at(name, i);
    const code = String.char_code_at(c, 0);
    const __lto_other_5 = 65;
    const __match_85 = (i < __lto_other_1);
    const __match_87 = ((__match_85 === true) ? Ordering["less"] : ((__match_85 === false) ? ((__match_86) => {
      if ((__match_86 === true)) {
        return Ordering["equal"];
      } else if ((__match_86 === false)) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_86);
      }
    })(((a, b) => {
      return (a === b);
    })(code, __lto_other_5)) : __lumo_match_error(__match_85)));
    const __match_88 = ((__match_87[LUMO_TAG] === "less") ? false : ((__match_87[LUMO_TAG] === "equal") ? true : ((__match_87[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_87))));
    const is_upper = ((__match_88 === true) ? ((__match_91) => {
      if ((__match_91[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_91[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return ((__match_91[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_91));
      }
    })(((__lto_self_8) => {
      const __lto_other_9 = 90;
      const __match_89 = (i < __lto_other_1);
      if ((__match_89 === true)) {
        return Ordering["less"];
      } else if ((__match_89 === false)) {
        const __match_90 = (i === __lto_other_1);
        if ((__match_90 === true)) {
          return Ordering["equal"];
        } else if ((__match_90 === false)) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_90);
        }
      } else {
        return __lumo_match_error(__match_89);
      }
    })(code)) : ((__match_88 === false) ? false : __lumo_match_error(__match_88)));
    if ((is_upper === true)) {
      let __match_68;
      let __match_67;
      const __lto_self_12 = 0;
      const __match_65 = (i < __lto_other_1);
      if ((__match_65 === true)) {
        __match_67 = Ordering["less"];
      } else if ((__match_65 === false)) {
        const __match_66 = (i === __lto_other_1);
        if ((__match_66 === true)) {
          __match_67 = Ordering["equal"];
        } else if ((__match_66 === false)) {
          __match_67 = Ordering["greater"];
        } else {
          __match_67 = __lumo_match_error(__match_66);
        }
      } else {
        __match_67 = __lumo_match_error(__match_65);
      }
      if ((__match_67[LUMO_TAG] === "less")) {
        __match_68 = true;
      } else if ((__match_67[LUMO_TAG] === "equal")) {
        __match_68 = false;
      } else {
        __match_68 = ((__match_67[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_67));
      }
      if ((__match_68 === true)) {
        const prev_code = String.char_code_at(String.char_at(name, ((__lto_self_16) => {
          const __lto_other_17 = 1;
          return (i - __lto_other_1);
        })(i)), 0);
        const __lto_other_21 = 97;
        const __match_78 = (i < __lto_other_1);
        const __match_80 = ((__match_78 === true) ? Ordering["less"] : ((__match_78 === false) ? ((__match_79) => {
          if ((__match_79 === true)) {
            return Ordering["equal"];
          } else if ((__match_79 === false)) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_79);
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, __lto_other_21)) : __lumo_match_error(__match_78)));
        const __match_81 = ((__match_80[LUMO_TAG] === "less") ? false : ((__match_80[LUMO_TAG] === "equal") ? true : ((__match_80[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_80))));
        const prev_lower = ((__match_81 === true) ? ((__match_84) => {
          if ((__match_84[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_84[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return ((__match_84[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_84));
          }
        })(((__lto_self_24) => {
          const __lto_other_25 = 122;
          const __match_82 = (i < __lto_other_1);
          if ((__match_82 === true)) {
            return Ordering["less"];
          } else if ((__match_82 === false)) {
            const __match_83 = (i === __lto_other_1);
            if ((__match_83 === true)) {
              return Ordering["equal"];
            } else if ((__match_83 === false)) {
              return Ordering["greater"];
            } else {
              return __lumo_match_error(__match_83);
            }
          } else {
            return __lumo_match_error(__match_82);
          }
        })(prev_code)) : ((__match_81 === false) ? false : __lumo_match_error(__match_81)));
        const __lto_other_29 = 48;
        const __match_71 = (prev_code < __lto_other_29);
        const __match_73 = ((__match_71 === true) ? Ordering["less"] : ((__match_71 === false) ? ((__match_72) => {
          if ((__match_72 === true)) {
            return Ordering["equal"];
          } else if ((__match_72 === false)) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_72);
          }
        })(((a, b) => {
          return (a === b);
        })(prev_code, __lto_other_29)) : __lumo_match_error(__match_71)));
        const __match_74 = ((__match_73[LUMO_TAG] === "less") ? false : ((__match_73[LUMO_TAG] === "equal") ? true : ((__match_73[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_73))));
        const prev_digit = ((__match_74 === true) ? ((__match_77) => {
          if ((__match_77[LUMO_TAG] === "less")) {
            return true;
          } else if ((__match_77[LUMO_TAG] === "equal")) {
            return true;
          } else {
            return ((__match_77[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_77));
          }
        })(((__lto_self_32) => {
          const __lto_other_33 = 57;
          const __match_75 = (i < __lto_other_1);
          if ((__match_75 === true)) {
            return Ordering["less"];
          } else if ((__match_75 === false)) {
            const __match_76 = (i === __lto_other_1);
            if ((__match_76 === true)) {
              return Ordering["equal"];
            } else if ((__match_76 === false)) {
              return Ordering["greater"];
            } else {
              return __lumo_match_error(__match_76);
            }
          } else {
            return __lumo_match_error(__match_75);
          }
        })(prev_code)) : ((__match_74 === false) ? false : __lumo_match_error(__match_74)));
        if ((prev_lower === true)) {
          return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_36) => {
            const __lto_other_37 = 1;
            return (i + __lto_other_1);
          })(i), ((__lto_self_40) => {
            const __lto_other_41 = to_upper_char__lto_f0f5f7cb(c);
            return (i + __lto_other_1);
          })(((__lto_self_42) => {
            const __lto_other_43 = "_";
            return (i + __lto_other_1);
          })(acc)));
        } else if ((prev_lower === false)) {
          if ((prev_digit === true)) {
            return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_48) => {
              const __lto_other_49 = 1;
              return (i + __lto_other_1);
            })(i), ((__lto_self_52) => {
              const __lto_other_53 = to_upper_char__lto_f0f5f7cb(c);
              return (i + __lto_other_1);
            })(((__lto_self_54) => {
              const __lto_other_55 = "_";
              return (i + __lto_other_1);
            })(acc)));
          } else if ((prev_digit === false)) {
            return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_60) => {
              const __lto_other_61 = 1;
              return (i + __lto_other_1);
            })(i), ((__lto_self_64) => {
              const __lto_other_65 = to_upper_char__lto_f0f5f7cb(c);
              return (i + __lto_other_1);
            })(acc));
          } else {
            return __lumo_match_error(prev_digit);
          }
        } else {
          return __lumo_match_error(prev_lower);
        }
      } else if ((__match_68 === false)) {
        return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_68) => {
          const __lto_other_69 = 1;
          return (i + __lto_other_1);
        })(i), ((__lto_self_72) => {
          const __lto_other_73 = to_upper_char__lto_f0f5f7cb(c);
          return (i + __lto_other_1);
        })(acc));
      } else {
        return __lumo_match_error(__match_68);
      }
    } else if ((is_upper === false)) {
      return to_screaming_snake_loop__lto_73ce111b(name, ((__lto_self_76) => {
        const __lto_other_77 = 1;
        return (i + __lto_other_1);
      })(i), ((__lto_self_80) => {
        const __lto_other_81 = to_upper_char__lto_f0f5f7cb(c);
        return (i + __lto_other_1);
      })(acc));
    } else {
      return __lumo_match_error(is_upper);
    }
  } else {
    return __lumo_match_error(__match_63);
  }
}

export function to_upper_char__lto_f0f5f7cb(c) {
  const code = String.char_code_at(c, 0);
  const __lto_other_85 = 97;
  const __match_92 = (code < __lto_other_85);
  const __match_94 = ((__match_92 === true) ? Ordering["less"] : ((__match_92 === false) ? ((__match_93) => {
    if ((__match_93 === true)) {
      return Ordering["equal"];
    } else if ((__match_93 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_93);
    }
  })(((a, b) => {
    return (a === b);
  })(code, __lto_other_85)) : __lumo_match_error(__match_92)));
  const __match_95 = ((__match_94[LUMO_TAG] === "less") ? false : ((__match_94[LUMO_TAG] === "equal") ? true : ((__match_94[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_94))));
  if ((__match_95 === true)) {
    let __match_99;
    let __match_98;
    const __lto_other_89 = 122;
    const __match_96 = (code < __lto_other_85);
    if ((__match_96 === true)) {
      __match_98 = Ordering["less"];
    } else if ((__match_96 === false)) {
      const __match_97 = (code === __lto_other_85);
      if ((__match_97 === true)) {
        __match_98 = Ordering["equal"];
      } else if ((__match_97 === false)) {
        __match_98 = Ordering["greater"];
      } else {
        __match_98 = __lumo_match_error(__match_97);
      }
    } else {
      __match_98 = __lumo_match_error(__match_96);
    }
    if ((__match_98[LUMO_TAG] === "less")) {
      __match_99 = true;
    } else if ((__match_98[LUMO_TAG] === "equal")) {
      __match_99 = true;
    } else {
      __match_99 = ((__match_98[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_98));
    }
    if ((__match_99 === true)) {
      const __lto_other_94 = 32;
      const __lto_code_92 = (code - __lto_other_85);
      return fromCharCode(__lto_code_92);
    } else if ((__match_99 === false)) {
      return c;
    } else {
      return __lumo_match_error(__match_99);
    }
  } else if ((__match_95 === false)) {
    return c;
  } else {
    return __lumo_match_error(__match_95);
  }
}

export function keyword_variant__lto_1ba4622a(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__lto_self_97) => {
    const __lto_other_98 = "_KW";
    return __k(((a, b) => {
      return (a + b);
    })(__lto_self_97, __lto_other_98));
  });
}

export function to_upper_string_loop__lto_1fab3ad0(s, i, acc) {
  const __lto_other_102 = String.len(s);
  const __match_100 = (i < __lto_other_102);
  const __match_102 = ((__match_100 === true) ? Ordering["less"] : ((__match_100 === false) ? ((__match_101) => {
    if ((__match_101 === true)) {
      return Ordering["equal"];
    } else if ((__match_101 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_101);
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_other_102)) : __lumo_match_error(__match_100)));
  const __match_103 = ((__match_102[LUMO_TAG] === "less") ? false : ((__match_102[LUMO_TAG] === "equal") ? true : ((__match_102[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_102))));
  if ((__match_103 === true)) {
    return acc;
  } else if ((__match_103 === false)) {
    return to_upper_string_loop__lto_1fab3ad0(s, ((__lto_self_105) => {
      const __lto_other_106 = 1;
      return (i + __lto_other_102);
    })(i), ((__lto_self_109) => {
      const __lto_other_110 = to_upper_char__lto_f0f5f7cb(String.char_at(s, i));
      return (i + __lto_other_102);
    })(acc));
  } else {
    return __lumo_match_error(__match_103);
  }
}

export function symbol_variant__lto_8227044e(sym) {
  const __lto_other_114 = "#";
  const __match_104 = (sym === __lto_other_114);
  if ((__match_104 === true)) {
    return "HASH";
  } else if ((__match_104 === false)) {
    const __lto_other_118 = "(";
    const __match_105 = (sym === __lto_other_114);
    if ((__match_105 === true)) {
      return "L_PAREN";
    } else if ((__match_105 === false)) {
      const __lto_other_122 = ")";
      const __match_106 = (sym === __lto_other_114);
      if ((__match_106 === true)) {
        return "R_PAREN";
      } else if ((__match_106 === false)) {
        const __lto_other_126 = "[";
        const __match_107 = (sym === __lto_other_114);
        if ((__match_107 === true)) {
          return "L_BRACKET";
        } else if ((__match_107 === false)) {
          const __lto_other_130 = "]";
          const __match_108 = (sym === __lto_other_114);
          if ((__match_108 === true)) {
            return "R_BRACKET";
          } else if ((__match_108 === false)) {
            const __lto_other_134 = "{";
            const __match_109 = (sym === __lto_other_114);
            if ((__match_109 === true)) {
              return "L_BRACE";
            } else if ((__match_109 === false)) {
              const __lto_other_138 = "}";
              const __match_110 = (sym === __lto_other_114);
              if ((__match_110 === true)) {
                return "R_BRACE";
              } else if ((__match_110 === false)) {
                const __lto_other_142 = ";";
                const __match_111 = (sym === __lto_other_114);
                if ((__match_111 === true)) {
                  return "SEMICOLON";
                } else if ((__match_111 === false)) {
                  const __lto_other_146 = ":";
                  const __match_112 = (sym === __lto_other_114);
                  if ((__match_112 === true)) {
                    return "COLON";
                  } else if ((__match_112 === false)) {
                    const __lto_other_150 = ",";
                    const __match_113 = (sym === __lto_other_114);
                    if ((__match_113 === true)) {
                      return "COMMA";
                    } else if ((__match_113 === false)) {
                      const __lto_other_154 = "=";
                      const __match_114 = (sym === __lto_other_114);
                      if ((__match_114 === true)) {
                        return "EQUALS";
                      } else if ((__match_114 === false)) {
                        const __lto_other_158 = ":=";
                        const __match_115 = (sym === __lto_other_114);
                        if ((__match_115 === true)) {
                          return "COLON_EQ";
                        } else if ((__match_115 === false)) {
                          const __lto_other_162 = "=>";
                          const __match_116 = (sym === __lto_other_114);
                          if ((__match_116 === true)) {
                            return "FAT_ARROW";
                          } else if ((__match_116 === false)) {
                            const __lto_other_166 = "->";
                            const __match_117 = (sym === __lto_other_114);
                            if ((__match_117 === true)) {
                              return "ARROW";
                            } else if ((__match_117 === false)) {
                              const __lto_other_170 = ".";
                              const __match_118 = (sym === __lto_other_114);
                              if ((__match_118 === true)) {
                                return "DOT";
                              } else if ((__match_118 === false)) {
                                const __lto_other_174 = "+";
                                const __match_119 = (sym === __lto_other_114);
                                if ((__match_119 === true)) {
                                  return "PLUS";
                                } else if ((__match_119 === false)) {
                                  const __lto_other_178 = "-";
                                  const __match_120 = (sym === __lto_other_114);
                                  if ((__match_120 === true)) {
                                    return "MINUS";
                                  } else if ((__match_120 === false)) {
                                    const __lto_other_182 = "*";
                                    const __match_121 = (sym === __lto_other_114);
                                    if ((__match_121 === true)) {
                                      return "STAR";
                                    } else if ((__match_121 === false)) {
                                      const __lto_other_186 = "/";
                                      const __match_122 = (sym === __lto_other_114);
                                      if ((__match_122 === true)) {
                                        return "SLASH";
                                      } else if ((__match_122 === false)) {
                                        const __lto_other_190 = "%";
                                        const __match_123 = (sym === __lto_other_114);
                                        if ((__match_123 === true)) {
                                          return "PERCENT";
                                        } else if ((__match_123 === false)) {
                                          const __lto_other_194 = "!";
                                          const __match_124 = (sym === __lto_other_114);
                                          if ((__match_124 === true)) {
                                            return "BANG";
                                          } else if ((__match_124 === false)) {
                                            const __lto_other_198 = "<";
                                            const __match_125 = (sym === __lto_other_114);
                                            if ((__match_125 === true)) {
                                              return "LT";
                                            } else if ((__match_125 === false)) {
                                              const __lto_other_202 = ">";
                                              const __match_126 = (sym === __lto_other_114);
                                              if ((__match_126 === true)) {
                                                return "GT";
                                              } else if ((__match_126 === false)) {
                                                const __lto_other_206 = "<=";
                                                const __match_127 = (sym === __lto_other_114);
                                                if ((__match_127 === true)) {
                                                  return "LT_EQ";
                                                } else if ((__match_127 === false)) {
                                                  const __lto_other_210 = ">=";
                                                  const __match_128 = (sym === __lto_other_114);
                                                  if ((__match_128 === true)) {
                                                    return "GT_EQ";
                                                  } else if ((__match_128 === false)) {
                                                    const __lto_other_214 = "==";
                                                    const __match_129 = (sym === __lto_other_114);
                                                    if ((__match_129 === true)) {
                                                      return "EQ_EQ";
                                                    } else if ((__match_129 === false)) {
                                                      const __lto_other_218 = "!=";
                                                      const __match_130 = (sym === __lto_other_114);
                                                      if ((__match_130 === true)) {
                                                        return "BANG_EQ";
                                                      } else if ((__match_130 === false)) {
                                                        const __lto_other_222 = "&&";
                                                        const __match_131 = (sym === __lto_other_114);
                                                        if ((__match_131 === true)) {
                                                          return "AMP_AMP";
                                                        } else if ((__match_131 === false)) {
                                                          const __lto_other_226 = "||";
                                                          const __match_132 = (sym === __lto_other_114);
                                                          if ((__match_132 === true)) {
                                                            return "PIPE_PIPE";
                                                          } else if ((__match_132 === false)) {
                                                            const __lto_other_230 = "_";
                                                            const __match_133 = (sym === __lto_other_114);
                                                            if ((__match_133 === true)) {
                                                              return "UNDERSCORE";
                                                            } else if ((__match_133 === false)) {
                                                              const __lto_self_233 = "SYM_";
                                                              return (sym + __lto_other_114);
                                                            } else {
                                                              return __lumo_match_error(__match_133);
                                                            }
                                                          } else {
                                                            return __lumo_match_error(__match_132);
                                                          }
                                                        } else {
                                                          return __lumo_match_error(__match_131);
                                                        }
                                                      } else {
                                                        return __lumo_match_error(__match_130);
                                                      }
                                                    } else {
                                                      return __lumo_match_error(__match_129);
                                                    }
                                                  } else {
                                                    return __lumo_match_error(__match_128);
                                                  }
                                                } else {
                                                  return __lumo_match_error(__match_127);
                                                }
                                              } else {
                                                return __lumo_match_error(__match_126);
                                              }
                                            } else {
                                              return __lumo_match_error(__match_125);
                                            }
                                          } else {
                                            return __lumo_match_error(__match_124);
                                          }
                                        } else {
                                          return __lumo_match_error(__match_123);
                                        }
                                      } else {
                                        return __lumo_match_error(__match_122);
                                      }
                                    } else {
                                      return __lumo_match_error(__match_121);
                                    }
                                  } else {
                                    return __lumo_match_error(__match_120);
                                  }
                                } else {
                                  return __lumo_match_error(__match_119);
                                }
                              } else {
                                return __lumo_match_error(__match_118);
                              }
                            } else {
                              return __lumo_match_error(__match_117);
                            }
                          } else {
                            return __lumo_match_error(__match_116);
                          }
                        } else {
                          return __lumo_match_error(__match_115);
                        }
                      } else {
                        return __lumo_match_error(__match_114);
                      }
                    } else {
                      return __lumo_match_error(__match_113);
                    }
                  } else {
                    return __lumo_match_error(__match_112);
                  }
                } else {
                  return __lumo_match_error(__match_111);
                }
              } else {
                return __lumo_match_error(__match_110);
              }
            } else {
              return __lumo_match_error(__match_109);
            }
          } else {
            return __lumo_match_error(__match_108);
          }
        } else {
          return __lumo_match_error(__match_107);
        }
      } else {
        return __lumo_match_error(__match_106);
      }
    } else {
      return __lumo_match_error(__match_105);
    }
  } else {
    return __lumo_match_error(__match_104);
  }
}

export function collect_tokens_from_alts__lto_9309ae26(__caps, alts, kws, syms, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return String.char_code_at(__caps, name, 0, (code) => {
          const __lto_other_238 = 65;
          const __match_141 = (code < __lto_other_238);
          const __match_143 = ((__match_141 === true) ? Ordering["less"] : ((__match_141 === false) ? ((__match_142) => {
            if ((__match_142 === true)) {
              return Ordering["equal"];
            } else if ((__match_142 === false)) {
              return Ordering["greater"];
            } else {
              return __lumo_match_error(__match_142);
            }
          })(((a, b) => {
            return (a === b);
          })(code, __lto_other_238)) : __lumo_match_error(__match_141)));
          const __match_136 = ((__match_143[LUMO_TAG] === "less") ? false : ((__match_143[LUMO_TAG] === "equal") ? true : ((__match_143[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_143))));
          if ((__match_136 === true)) {
            const __lto_other_242 = 90;
            const __match_138 = (code < __lto_other_238);
            const __match_140 = ((__match_138 === true) ? Ordering["less"] : ((__match_138 === false) ? ((__match_139) => {
              if ((__match_139 === true)) {
                return Ordering["equal"];
              } else if ((__match_139 === false)) {
                return Ordering["greater"];
              } else {
                return __lumo_match_error(__match_139);
              }
            })(((a, b) => {
              return (a === b);
            })(code, __lto_other_242)) : __lumo_match_error(__match_138)));
            const __match_137 = ((__match_140[LUMO_TAG] === "less") ? true : ((__match_140[LUMO_TAG] === "equal") ? true : ((__match_140[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_140))));
            if ((__match_137 === true)) {
              return collect_tokens_from_alts__lto_9309ae26(__caps, rest, kws, syms, __k);
            } else if ((__match_137 === false)) {
              return collect_alt_token(__caps, name, rest, kws, syms, __k);
            } else {
              return __lumo_match_error(__match_137);
            }
          } else if ((__match_136 === false)) {
            return collect_alt_token(__caps, name, rest, kws, syms, __k);
          } else {
            return __lumo_match_error(__match_136);
          }
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function string_lt_loop__lto_090deca7(a, b, i) {
  const __lto_other_246 = String.len(a);
  const __match_144 = (i < __lto_other_246);
  const __match_146 = ((__match_144 === true) ? Ordering["less"] : ((__match_144 === false) ? ((__match_145) => {
    if ((__match_145 === true)) {
      return Ordering["equal"];
    } else if ((__match_145 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_145);
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_other_246)) : __lumo_match_error(__match_144)));
  const __match_147 = ((__match_146[LUMO_TAG] === "less") ? false : ((__match_146[LUMO_TAG] === "equal") ? true : ((__match_146[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_146))));
  if ((__match_147 === true)) {
    let __match_163;
    let __match_162;
    const __lto_other_250 = String.len(b);
    const __match_160 = (i < __lto_other_246);
    if ((__match_160 === true)) {
      __match_162 = Ordering["less"];
    } else if ((__match_160 === false)) {
      const __match_161 = (i === __lto_other_246);
      if ((__match_161 === true)) {
        __match_162 = Ordering["equal"];
      } else if ((__match_161 === false)) {
        __match_162 = Ordering["greater"];
      } else {
        __match_162 = __lumo_match_error(__match_161);
      }
    } else {
      __match_162 = __lumo_match_error(__match_160);
    }
    if ((__match_162[LUMO_TAG] === "less")) {
      __match_163 = false;
    } else if ((__match_162[LUMO_TAG] === "equal")) {
      __match_163 = true;
    } else {
      __match_163 = ((__match_162[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_162));
    }
    if ((__match_163 === true)) {
      return false;
    } else if ((__match_163 === false)) {
      return true;
    } else {
      return __lumo_match_error(__match_163);
    }
  } else if ((__match_147 === false)) {
    let __match_151;
    let __match_150;
    const __lto_other_254 = String.len(b);
    const __match_148 = (i < __lto_other_246);
    if ((__match_148 === true)) {
      __match_150 = Ordering["less"];
    } else if ((__match_148 === false)) {
      const __match_149 = (i === __lto_other_246);
      if ((__match_149 === true)) {
        __match_150 = Ordering["equal"];
      } else if ((__match_149 === false)) {
        __match_150 = Ordering["greater"];
      } else {
        __match_150 = __lumo_match_error(__match_149);
      }
    } else {
      __match_150 = __lumo_match_error(__match_148);
    }
    if ((__match_150[LUMO_TAG] === "less")) {
      __match_151 = false;
    } else if ((__match_150[LUMO_TAG] === "equal")) {
      __match_151 = true;
    } else {
      __match_151 = ((__match_150[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_150));
    }
    if ((__match_151 === true)) {
      return false;
    } else if ((__match_151 === false)) {
      const ca = String.char_code_at(i, i);
      const cb = String.char_code_at(b, i);
      const __match_152 = (i < __lto_other_246);
      const __match_154 = ((__match_152 === true) ? Ordering["less"] : ((__match_152 === false) ? ((__match_153) => {
        if ((__match_153 === true)) {
          return Ordering["equal"];
        } else if ((__match_153 === false)) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_153);
        }
      })(((a, b) => {
        return (a === b);
      })(ca, cb)) : __lumo_match_error(__match_152)));
      const __match_155 = ((__match_154[LUMO_TAG] === "less") ? true : ((__match_154[LUMO_TAG] === "equal") ? false : ((__match_154[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_154))));
      if ((__match_155 === true)) {
        return true;
      } else if ((__match_155 === false)) {
        let __match_159;
        let __match_158;
        const __match_156 = (i < __lto_other_246);
        if ((__match_156 === true)) {
          __match_158 = Ordering["less"];
        } else if ((__match_156 === false)) {
          const __match_157 = (i === __lto_other_246);
          if ((__match_157 === true)) {
            __match_158 = Ordering["equal"];
          } else if ((__match_157 === false)) {
            __match_158 = Ordering["greater"];
          } else {
            __match_158 = __lumo_match_error(__match_157);
          }
        } else {
          __match_158 = __lumo_match_error(__match_156);
        }
        if ((__match_158[LUMO_TAG] === "less")) {
          __match_159 = true;
        } else if ((__match_158[LUMO_TAG] === "equal")) {
          __match_159 = false;
        } else {
          __match_159 = ((__match_158[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_158));
        }
        if ((__match_159 === true)) {
          return false;
        } else if ((__match_159 === false)) {
          return string_lt_loop__lto_090deca7(i, __lto_other_246, ((__lto_self_265) => {
            const __lto_other_266 = 1;
            return (__lto_self_265 + __lto_other_266);
          })(i));
        } else {
          return __lumo_match_error(__match_159);
        }
      } else {
        return __lumo_match_error(__match_155);
      }
    } else {
      return __lumo_match_error(__match_151);
    }
  } else {
    return __lumo_match_error(__match_147);
  }
}

export function is_token_only_alternatives__lto_9309ae26(alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return true;
  } else if ((alts[LUMO_TAG] === "cons")) {
    const alt = alts.args[0];
    const rest = alts.args[1];
    if ((alt[LUMO_TAG] === "mk")) {
      const name = alt.args[0];
      const code = String.char_code_at(name, 0);
      const __lto_other_270 = 65;
      const __match_167 = (code < __lto_other_270);
      const __match_169 = ((__match_167 === true) ? Ordering["less"] : ((__match_167 === false) ? ((__match_168) => {
        if ((__match_168 === true)) {
          return Ordering["equal"];
        } else if ((__match_168 === false)) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_168);
        }
      })(((a, b) => {
        return (a === b);
      })(code, __lto_other_270)) : __lumo_match_error(__match_167)));
      const __match_170 = ((__match_169[LUMO_TAG] === "less") ? false : ((__match_169[LUMO_TAG] === "equal") ? true : ((__match_169[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_169))));
      const is_upper = ((__match_170 === true) ? ((__match_173) => {
        if ((__match_173[LUMO_TAG] === "less")) {
          return true;
        } else if ((__match_173[LUMO_TAG] === "equal")) {
          return true;
        } else {
          return ((__match_173[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_173));
        }
      })(((__lto_self_273) => {
        const __lto_other_274 = 90;
        const __match_171 = (code < __lto_other_270);
        if ((__match_171 === true)) {
          return Ordering["less"];
        } else if ((__match_171 === false)) {
          const __match_172 = (code === __lto_other_270);
          if ((__match_172 === true)) {
            return Ordering["equal"];
          } else if ((__match_172 === false)) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_172);
          }
        } else {
          return __lumo_match_error(__match_171);
        }
      })(code)) : ((__match_170 === false) ? false : __lumo_match_error(__match_170)));
      if ((is_upper === true)) {
        return false;
      } else if ((is_upper === false)) {
        return is_token_only_alternatives__lto_9309ae26(rest);
      } else {
        return __lumo_match_error(is_upper);
      }
    } else {
      return __lumo_match_error(alt);
    }
  } else {
    return __lumo_match_error(alts);
  }
}

export function generate_syntax_kind__lto_1ba4622a(__caps, grammar, __k) {
  return collect_tokens(__caps, grammar, (collected) => {
    if ((collected[LUMO_TAG] === "mk")) {
      const keywords = collected.args[0];
      const symbols = collected.args[1];
      if ((grammar[LUMO_TAG] === "mk")) {
        const token_defs = grammar.args[0];
        const rules = grammar.args[1];
        const s = "// Auto-generated by langue. Do not edit.\n";
        const __lto_other_278 = "// Regenerate: scripts/gen_langue.sh\n\n";
        const s_0 = (s + __lto_other_278);
        const __lto_other_282 = "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n";
        const s_1 = (s_0 + __lto_other_282);
        const __lto_other_286 = "#[repr(u16)]\n";
        const s_4 = (s_1 + __lto_other_286);
        const __lto_other_290 = "pub enum SyntaxKind {\n";
        const s_7 = (s_4 + __lto_other_290);
        const __lto_other_294 = "    // Named tokens\n";
        const s_10 = (s_7 + __lto_other_294);
        return emit_named_tokens__lto_1ba4622a(__caps, s_10, token_defs, (s) => {
          const __lto_other_298 = "    // Trivia\n";
          const s_0 = (s + __lto_other_278);
          const __lto_other_302 = "    WHITESPACE,\n    NEWLINE,\n    UNKNOWN,\n";
          const s_1 = (s_0 + __lto_other_282);
          return emit_keywords__lto_1ba4622a(__caps, s_1, keywords, (s) => {
            const s__iife_0 = emit_symbols__lto_1ba4622a(s, symbols);
            const __lto_other_306 = "    // Nodes\n";
            const s_0 = (s + __lto_other_278);
            return emit_node_kinds__lto_1ba4622a(__caps, s_0, rules, (s) => {
              const __lto_other_310 = "    // Sentinel\n    ERROR,\n";
              const s_0 = (s + __lto_other_278);
              const __lto_other_314 = "}\n";
              const s_1 = (s_0 + __lto_other_282);
              const __lto_other_318 = "\nimpl SyntaxKind {\n";
              const s_4 = (s_1 + __lto_other_286);
              const __lto_other_322 = "    pub fn is_trivia(self) -> bool {\n";
              const s_7 = (s_4 + __lto_other_290);
              const __lto_other_326 = "        matches!(self, Self::WHITESPACE | Self::NEWLINE)\n";
              const s_10 = (s_7 + __lto_other_294);
              const __lto_other_330 = "    }\n";
              const s_13 = (s_10 + __lto_other_330);
              return emit_from_keyword__lto_1ba4622a(__caps, s_13, keywords, (s) => {
                const s__iife_1 = emit_from_symbol__lto_1ba4622a(s, symbols);
                const __lto_other_334 = "}\n";
                const s_0 = (s + __lto_other_278);
                return __k(s_0);
              });
            });
          });
        });
      } else {
        return __lumo_match_error(grammar);
      }
    } else {
      return __lumo_match_error(collected);
    }
  });
}

export function emit_named_tokens__lto_1ba4622a(__caps, s, tokens, __k) {
  return __thunk(() => {
    if ((tokens[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((tokens[LUMO_TAG] === "cons")) {
      const tok = tokens.args[0];
      const rest = tokens.args[1];
      const __lto_other_342 = "    ";
      const __lto_self_339 = (s + __lto_other_342);
      return to_screaming_snake(__caps, tok, (__lto_other_340) => {
        const __lto_self_337 = (s + __lto_other_342);
        const __lto_other_338 = ",\n";
        const __cps_v_31 = (__lto_self_337 + __lto_other_338);
        return emit_named_tokens__lto_1ba4622a(__caps, __cps_v_31, rest, __k);
      });
    } else {
      return __lumo_match_error(tokens);
    }
  });
}

export function emit_keywords__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __lto_other_350 = "    // Keywords\n";
      const s2 = (s + __lto_other_350);
      return emit_keywords_items__lto_1ba4622a(__caps, s2, kws, __k);
    }
  });
}

export function emit_keywords_items__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      const __lto_self_359 = "    ";
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_360) => {
        const __lto_self_357 = (__lto_self_359 + __lto_other_360);
        const __lto_other_358 = ", // '";
        const __lto_self_355 = (__lto_self_357 + __lto_other_358);
        const __lto_self_353 = (__lto_self_355 + kw);
        const __lto_other_354 = "'\n";
        const line = (__lto_self_353 + __lto_other_354);
        return emit_keywords_items__lto_1ba4622a(__caps, ((__lto_self_369) => {
          return (__lto_self_359 + __lto_other_360);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_symbols__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else if ((syms[LUMO_TAG] === "cons")) {
    const sym = syms.args[0];
    const rest = syms.args[1];
    const s2 = ((rest[LUMO_TAG] === "nil") ? s : ((__match_181) => {
      const __lto_other_374 = "    // Symbols\n";
      return (s + __lto_other_374);
    })(syms));
    return emit_symbols_items__lto_1ba4622a(s2, syms);
  } else {
    return __lumo_match_error(syms);
  }
}

export function emit_symbols_items__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else if ((syms[LUMO_TAG] === "cons")) {
    const sym = syms.args[0];
    const rest = syms.args[1];
    const __lto_self_383 = "    ";
    const __lto_other_384 = symbol_variant__lto_8227044e(sym);
    const __lto_self_381 = (__lto_self_383 + __lto_other_384);
    const __lto_other_382 = ", // '";
    const __lto_self_379 = (__lto_self_381 + __lto_other_382);
    const __lto_self_377 = (__lto_self_379 + sym);
    const __lto_other_378 = "'\n";
    const line = (__lto_self_377 + __lto_other_378);
    return emit_symbols_items__lto_1ba4622a(((__lto_self_393) => {
      return (__lto_self_383 + __lto_other_384);
    })(s), rest);
  } else {
    return __lumo_match_error(syms);
  }
}

export function emit_node_kinds__lto_1ba4622a(__caps, s, rules, __k) {
  return __thunk(() => {
    if ((rules[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((rules[LUMO_TAG] === "cons")) {
      const rule = rules.args[0];
      const rest = rules.args[1];
      if ((rule[LUMO_TAG] === "mk")) {
        const name = rule.args[0];
        const body = rule.args[1];
        if ((body[LUMO_TAG] === "sequence")) {
          const elems = body.args[0];
          const __lto_self_403 = "    ";
          return to_screaming_snake(__caps, name, (__lto_other_404) => {
            const __lto_self_401 = (__lto_self_403 + __lto_other_404);
            const __lto_other_402 = ", // ";
            const __lto_self_399 = (__lto_self_401 + __lto_other_402);
            const __lto_self_397 = (__lto_self_399 + name);
            const __lto_other_398 = "\n";
            const line = (__lto_self_397 + __lto_other_398);
            return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_413) => {
              return (__lto_self_403 + __lto_other_404);
            })(s), rest, __k);
          });
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          const __match_186 = is_token_only_alternatives__lto_9309ae26(alts);
          if ((__match_186 === true)) {
            const __lto_self_423 = "    ";
            return to_screaming_snake(__caps, name, (__lto_other_424) => {
              const __lto_self_421 = (__lto_self_423 + __lto_other_424);
              const __lto_other_422 = ", // ";
              const __lto_self_419 = (__lto_self_421 + __lto_other_422);
              const __lto_self_417 = (__lto_self_419 + name);
              const __lto_other_418 = " (token wrapper)\n";
              const line = (__lto_self_417 + __lto_other_418);
              return emit_node_kinds__lto_1ba4622a(__caps, ((__lto_self_433) => {
                return (__lto_self_423 + __lto_other_424);
              })(s), rest, __k);
            });
          } else if ((__match_186 === false)) {
            return emit_node_kinds__lto_1ba4622a(__caps, s, rest, __k);
          } else {
            return __lumo_match_error(__match_186);
          }
        } else {
          return __lumo_match_error(body);
        }
      } else {
        return __lumo_match_error(rule);
      }
    } else {
      return __lumo_match_error(rules);
    }
  });
}

export function emit_from_keyword__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __lto_other_438 = "\n    pub fn from_keyword(text: &str) -> Option<Self> {\n";
      const s = (s + __lto_other_438);
      const __lto_other_442 = "        match text {\n";
      const s_0 = (s + __lto_other_442);
      return emit_keyword_arms__lto_1ba4622a(__caps, s_0, kws, (s) => {
        const __lto_other_446 = "            _ => None,\n";
        const s_0 = (s + __lto_other_438);
        const __lto_other_450 = "        }\n";
        const s_1 = (s_0 + __lto_other_450);
        const __lto_other_454 = "    }\n";
        const s_4 = (s_1 + __lto_other_454);
        return __k(s_4);
      });
    }
  });
}

export function emit_keyword_arms__lto_1ba4622a(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      const __lto_self_463 = "            \"";
      const __lto_self_461 = (__lto_self_463 + kw);
      const __lto_other_462 = "\" => Some(Self::";
      const __lto_self_459 = (__lto_self_461 + __lto_other_462);
      return keyword_variant__lto_1ba4622a(__caps, kw, (__lto_other_460) => {
        const __lto_self_457 = (__lto_self_463 + kw);
        const __lto_other_458 = "),\n";
        const line = (__lto_self_461 + __lto_other_462);
        return emit_keyword_arms__lto_1ba4622a(__caps, ((__lto_self_473) => {
          return (__lto_self_463 + kw);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_from_symbol__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else {
    const __lto_other_478 = "\n    pub fn from_symbol(text: &str) -> Option<Self> {\n";
    const s__iife_18 = (s + __lto_other_478);
    const __lto_other_482 = "        match text {\n";
    const s__iife_2 = (s__iife_18 + __lto_other_482);
    const s_0 = emit_symbol_arms__lto_1ba4622a(s__iife_2, syms);
    const __lto_other_486 = "            _ => None,\n";
    const s__iife_3 = (s_0 + __lto_other_486);
    const __lto_other_490 = "        }\n";
    const s_3 = (s__iife_3 + __lto_other_490);
    const __lto_other_494 = "    }\n";
    const s__iife_4 = (s_3 + __lto_other_494);
    return s__iife_4;
  }
}

export function emit_symbol_arms__lto_1ba4622a(s, syms) {
  if ((syms[LUMO_TAG] === "nil")) {
    return s;
  } else if ((syms[LUMO_TAG] === "cons")) {
    const sym = syms.args[0];
    const rest = syms.args[1];
    const __lto_self_503 = "            \"";
    const __lto_self_501 = (__lto_self_503 + sym);
    const __lto_other_502 = "\" => Some(Self::";
    const __lto_self_499 = (__lto_self_501 + __lto_other_502);
    const __lto_other_500 = symbol_variant__lto_8227044e(sym);
    const __lto_self_497 = (__lto_self_499 + __lto_other_500);
    const __lto_other_498 = "),\n";
    const line = (__lto_self_497 + __lto_other_498);
    return emit_symbol_arms__lto_1ba4622a(((__lto_self_513) => {
      return (__lto_self_503 + sym);
    })(s), rest);
  } else {
    return __lumo_match_error(syms);
  }
}

export function generate_ast__lto_1ba4622a(__caps, grammar, __k) {
  return __thunk(() => {
    if ((grammar[LUMO_TAG] === "mk")) {
      const token_defs = grammar.args[0];
      const rules = grammar.args[1];
      const s = "// Auto-generated by langue. Do not edit.\n";
      const __lto_other_518 = "// Regenerate: scripts/gen_langue.sh\n\n";
      const s_0 = (s + __lto_other_518);
      const __lto_other_522 = "use super::SyntaxKind;\n";
      const s_1 = (s_0 + __lto_other_522);
      const __lto_other_526 = "use super::{SyntaxNode, SyntaxElement, LosslessToken};\n\n";
      const s_4 = (s_1 + __lto_other_526);
      const __lto_other_530 = "pub trait AstNode<'a>: Sized {\n";
      const s_7 = (s_4 + __lto_other_530);
      const __lto_other_534 = "    fn cast(node: &'a SyntaxNode) -> Option<Self>;\n";
      const s_10 = (s_7 + __lto_other_534);
      const __lto_other_538 = "    fn syntax(&self) -> &'a SyntaxNode;\n";
      const s_13 = (s_10 + __lto_other_538);
      const __lto_other_542 = "}\n\n";
      const s_16 = (s_13 + __lto_other_542);
      return emit_ast_rules(__caps, s_16, token_defs, rules, __k);
    } else {
      return __lumo_match_error(grammar);
    }
  });
}

export function emit_struct_node__lto_1ba4622a(__caps, s, name, elems, token_defs, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    const __lto_other_550 = "pub struct ";
    const __lto_self_547 = (s + __lto_other_550);
    const __lto_self_545 = (__lto_self_547 + name);
    const __lto_other_546 = "<'a>(pub(crate) &'a SyntaxNode);\n\n";
    const s = (__lto_self_545 + __lto_other_546);
    const __lto_other_562 = "impl<'a> AstNode<'a> for ";
    const __lto_self_559 = (s + __lto_other_562);
    const __lto_self_557 = (__lto_self_559 + name);
    const __lto_other_558 = "<'a> {\n";
    const s_8 = (__lto_self_557 + __lto_other_558);
    const __lto_other_570 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
    const s_11 = (s_8 + __lto_other_570);
    const __lto_other_578 = "        (node.kind == SyntaxKind::";
    const __lto_self_575 = (s_11 + __lto_other_578);
    const __lto_self_573 = (__lto_self_575 + kind);
    const __lto_other_574 = ").then(|| Self(node))\n";
    const s_18 = (__lto_self_573 + __lto_other_574);
    const __lto_other_586 = "    }\n";
    const s_21 = (s_18 + __lto_other_586);
    const __lto_other_590 = "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n";
    const s_24 = (s_21 + __lto_other_590);
    const __lto_other_594 = "}\n\n";
    const s_27 = (s_24 + __lto_other_594);
    return emit_accessors__lto_1ba4622a(__caps, s_27, name, elems, token_defs, __k);
  });
}

export function emit_accessors__lto_1ba4622a(__caps, s, struct_name, elems, token_defs, __k) {
  return __thunk(() => {
    const has_labeled = has_labeled_elements(elems);
    if ((has_labeled === true)) {
      const __lto_other_602 = "impl<'a> ";
      const __lto_self_599 = (s + __lto_other_602);
      const __lto_self_597 = (__lto_self_599 + struct_name);
      const __lto_other_598 = "<'a> {\n";
      const s = (__lto_self_597 + __lto_other_598);
      return emit_accessors_for_elements(__caps, s, elems, token_defs, (s) => {
        const __lto_other_610 = "}\n\n";
        const s_0 = (s + __lto_other_602);
        return __k(s_0);
      });
    } else if ((has_labeled === false)) {
      return __k(s);
    } else {
      return __lumo_match_error(has_labeled);
    }
  });
}

export function emit_token_accessor__lto_1ba4622a(__caps, s, label, t, repeated, __k) {
  return token_kind_from_ref(__caps, t, (kind) => {
    if ((repeated === true)) {
      const __lto_other_618 = "    pub fn ";
      const __lto_self_615 = (s + __lto_other_618);
      const __lto_self_613 = (__lto_self_615 + label);
      const __lto_other_614 = "(&self) -> impl Iterator<Item = &'a LosslessToken> + 'a {\n";
      const s = (__lto_self_613 + __lto_other_614);
      const __lto_other_626 = "        self.0.children.iter().filter_map(|c| match c {\n";
      const s_4 = (s + __lto_other_626);
      const __lto_other_634 = "            SyntaxElement::Token(t) if t.kind == SyntaxKind::";
      const __lto_self_631 = (s_4 + __lto_other_634);
      const __lto_self_629 = (__lto_self_631 + kind);
      const __lto_other_630 = " => Some(t),\n";
      const s_11 = (__lto_self_629 + __lto_other_630);
      const __lto_other_642 = "            _ => None,\n";
      const s_14 = (s_11 + __lto_other_642);
      const __lto_other_646 = "        })\n";
      const s_17 = (s_14 + __lto_other_646);
      const __lto_other_650 = "    }\n";
      return __k(((a, b) => {
        return (a + b);
      })(s_17, __lto_other_650));
    } else if ((repeated === false)) {
      const __lto_other_658 = "    pub fn ";
      const __lto_self_655 = (s + __lto_other_658);
      const __lto_self_653 = (__lto_self_655 + label);
      const __lto_other_654 = "(&self) -> Option<&'a LosslessToken> {\n";
      const s = (__lto_self_653 + __lto_other_654);
      const __lto_other_666 = "        self.0.children.iter().find_map(|c| match c {\n";
      const s_4 = (s + __lto_other_666);
      const __lto_other_674 = "            SyntaxElement::Token(t) if t.kind == SyntaxKind::";
      const __lto_self_671 = (s_4 + __lto_other_674);
      const __lto_self_669 = (__lto_self_671 + kind);
      const __lto_other_670 = " => Some(t),\n";
      const s_11 = (__lto_self_669 + __lto_other_670);
      const __lto_other_682 = "            _ => None,\n";
      const s_14 = (s_11 + __lto_other_682);
      const __lto_other_686 = "        })\n";
      const s_17 = (s_14 + __lto_other_686);
      const __lto_other_690 = "    }\n";
      return __k(((a, b) => {
        return (a + b);
      })(s_17, __lto_other_690));
    } else {
      return __lumo_match_error(repeated);
    }
  });
}

export function emit_node_accessor__lto_1ba4622a(s, label, node_name, repeated) {
  if ((repeated === true)) {
    const __lto_other_702 = "    pub fn ";
    const __lto_self_699 = (s + __lto_other_702);
    const __lto_self_697 = (__lto_self_699 + label);
    const __lto_other_698 = "(&self) -> impl Iterator<Item = ";
    const __lto_self_695 = (__lto_self_697 + __lto_other_698);
    const __lto_self_693 = (__lto_self_695 + node_name);
    const __lto_other_694 = "<'a>> + 'a {\n";
    const s__iife_19 = (__lto_self_693 + __lto_other_694);
    const __lto_other_714 = "        self.0.children.iter().filter_map(|c| match c {\n";
    const s__iife_5 = (s__iife_19 + __lto_other_714);
    const __lto_other_722 = "            SyntaxElement::Node(n) => ";
    const __lto_self_719 = (s__iife_5 + __lto_other_722);
    const __lto_self_717 = (__lto_self_719 + node_name);
    const __lto_other_718 = "::cast(n),\n";
    const s_0 = (__lto_self_717 + __lto_other_718);
    const __lto_other_730 = "            _ => None,\n";
    const s__iife_6 = (s_0 + __lto_other_730);
    const __lto_other_734 = "        })\n";
    const s_9 = (s__iife_6 + __lto_other_734);
    const __lto_other_738 = "    }\n";
    return (s_9 + __lto_other_738);
  } else if ((repeated === false)) {
    const __lto_other_750 = "    pub fn ";
    const __lto_self_747 = (s + __lto_other_750);
    const __lto_self_745 = (__lto_self_747 + label);
    const __lto_other_746 = "(&self) -> Option<";
    const __lto_self_743 = (__lto_self_745 + __lto_other_746);
    const __lto_self_741 = (__lto_self_743 + node_name);
    const __lto_other_742 = "<'a>> {\n";
    const s__iife_20 = (__lto_self_741 + __lto_other_742);
    const __lto_other_762 = "        self.0.children.iter().find_map(|c| match c {\n";
    const s__iife_7 = (s__iife_20 + __lto_other_762);
    const __lto_other_770 = "            SyntaxElement::Node(n) => ";
    const __lto_self_767 = (s__iife_7 + __lto_other_770);
    const __lto_self_765 = (__lto_self_767 + node_name);
    const __lto_other_766 = "::cast(n),\n";
    const s_0 = (__lto_self_765 + __lto_other_766);
    const __lto_other_778 = "            _ => None,\n";
    const s__iife_8 = (s_0 + __lto_other_778);
    const __lto_other_782 = "        })\n";
    const s_9 = (s__iife_8 + __lto_other_782);
    const __lto_other_786 = "    }\n";
    return (s_9 + __lto_other_786);
  } else {
    return __lumo_match_error(repeated);
  }
}

export function emit_enum_node__lto_1ba4622a(s, name, alts) {
  const __lto_other_794 = "pub enum ";
  const __lto_self_791 = (s + __lto_other_794);
  const __lto_self_789 = (__lto_self_791 + name);
  const __lto_other_790 = "<'a> {\n";
  const s__iife_9 = (__lto_self_789 + __lto_other_790);
  const s_4 = emit_enum_variants__lto_1ba4622a(s__iife_9, alts);
  const __lto_other_802 = "}\n\n";
  const s__iife_10 = (s_4 + __lto_other_802);
  const __lto_other_810 = "impl<'a> AstNode<'a> for ";
  const __lto_self_807 = (s__iife_10 + __lto_other_810);
  const __lto_self_805 = (__lto_self_807 + name);
  const __lto_other_806 = "<'a> {\n";
  const s_7 = (__lto_self_805 + __lto_other_806);
  const __lto_other_818 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
  const s__iife_11 = (s_7 + __lto_other_818);
  const __lto_other_822 = "        None\n";
  const s_16 = (s__iife_11 + __lto_other_822);
  const s__iife_12 = emit_enum_cast_chain__lto_1ba4622a(s_16, alts);
  const __lto_other_826 = "    }\n";
  const s_19 = (s__iife_12 + __lto_other_826);
  const __lto_other_830 = "    fn syntax(&self) -> &'a SyntaxNode {\n";
  const s__iife_13 = (s_19 + __lto_other_830);
  const __lto_other_834 = "        match self {\n";
  const s_24 = (s__iife_13 + __lto_other_834);
  const s__iife_14 = emit_enum_syntax_arms__lto_1ba4622a(s_24, alts);
  const __lto_other_838 = "        }\n";
  const s_27 = (s__iife_14 + __lto_other_838);
  const __lto_other_842 = "    }\n";
  const s__iife_15 = (s_27 + __lto_other_842);
  const __lto_other_846 = "}\n\n";
  const s_32 = (s__iife_15 + __lto_other_846);
  return s_32;
}

export function emit_enum_variants__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else if ((alts[LUMO_TAG] === "cons")) {
    const alt = alts.args[0];
    const rest = alts.args[1];
    if ((alt[LUMO_TAG] === "mk")) {
      const name = alt.args[0];
      return emit_enum_variants__lto_1ba4622a(((__lto_self_849) => {
        const __lto_other_850 = "<'a>),\n";
        return (__lto_self_849 + __lto_other_850);
      })(((__lto_self_851) => {
        return (__lto_self_851 + name);
      })(((__lto_self_853) => {
        const __lto_other_854 = "(";
        return (__lto_self_853 + __lto_other_854);
      })(((__lto_self_855) => {
        return (__lto_self_855 + name);
      })(((__lto_self_857) => {
        const __lto_other_858 = "    ";
        return (__lto_self_857 + __lto_other_858);
      })(s))))), rest);
    } else {
      return __lumo_match_error(alt);
    }
  } else {
    return __lumo_match_error(alts);
  }
}

export function emit_enum_cast_chain__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else if ((alts[LUMO_TAG] === "cons")) {
    const alt = alts.args[0];
    const rest = alts.args[1];
    if ((alt[LUMO_TAG] === "mk")) {
      const name = alt.args[0];
      const __lto_self_875 = "            .or_else(|| ";
      const __lto_self_873 = (__lto_self_875 + name);
      const __lto_other_874 = "::cast(node).map(Self::";
      const __lto_self_871 = (__lto_self_873 + __lto_other_874);
      const __lto_self_869 = (__lto_self_871 + name);
      const __lto_other_870 = "))\n";
      const line = (__lto_self_869 + __lto_other_870);
      return emit_enum_cast_chain__lto_1ba4622a(((__lto_self_885) => {
        return (__lto_self_875 + name);
      })(s), rest);
    } else {
      return __lumo_match_error(alt);
    }
  } else {
    return __lumo_match_error(alts);
  }
}

export function emit_enum_syntax_arms__lto_1ba4622a(s, alts) {
  if ((alts[LUMO_TAG] === "nil")) {
    return s;
  } else if ((alts[LUMO_TAG] === "cons")) {
    const alt = alts.args[0];
    const rest = alts.args[1];
    if ((alt[LUMO_TAG] === "mk")) {
      const name = alt.args[0];
      const __lto_self_891 = "            Self::";
      const __lto_self_889 = (__lto_self_891 + name);
      const __lto_other_890 = "(n) => n.syntax(),\n";
      const line = (__lto_self_889 + __lto_other_890);
      return emit_enum_syntax_arms__lto_1ba4622a(((__lto_self_897) => {
        return (__lto_self_891 + name);
      })(s), rest);
    } else {
      return __lumo_match_error(alt);
    }
  } else {
    return __lumo_match_error(alts);
  }
}

export function emit_token_wrapper_node__lto_1ba4622a(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    const __lto_other_906 = "pub struct ";
    const __lto_self_903 = (s + __lto_other_906);
    const __lto_self_901 = (__lto_self_903 + name);
    const __lto_other_902 = "<'a>(pub(crate) &'a SyntaxNode);\n\n";
    const s = (__lto_self_901 + __lto_other_902);
    const __lto_other_918 = "impl<'a> AstNode<'a> for ";
    const __lto_self_915 = (s + __lto_other_918);
    const __lto_self_913 = (__lto_self_915 + name);
    const __lto_other_914 = "<'a> {\n";
    const s_8 = (__lto_self_913 + __lto_other_914);
    const __lto_other_926 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
    const s_11 = (s_8 + __lto_other_926);
    const __lto_other_934 = "        (node.kind == SyntaxKind::";
    const __lto_self_931 = (s_11 + __lto_other_934);
    const __lto_self_929 = (__lto_self_931 + kind);
    const __lto_other_930 = ").then(|| Self(node))\n";
    const s_18 = (__lto_self_929 + __lto_other_930);
    const __lto_other_942 = "    }\n";
    const s_21 = (s_18 + __lto_other_942);
    const __lto_other_946 = "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n";
    const s_24 = (s_21 + __lto_other_946);
    const __lto_other_950 = "}\n\n";
    const s_27 = (s_24 + __lto_other_950);
    return __k(s_27);
  });
}

export function run__lto_3829b133(__caps, __k) {
  return __thunk(() => {
    const __lto___lto_self_1221_1225 = __argv_length_raw();
    const __lto___lto_other_1222_1226 = 1;
    const __lto_self_953 = (__lto___lto_self_1221_1225 - __lto___lto_other_1222_1226);
    const __lto_other_954 = 2;
    const __match_204 = (__lto_self_953 < __lto_other_954);
    const __match_206 = ((__match_204 === true) ? Ordering["less"] : ((__match_204 === false) ? ((__match_205) => {
      if ((__match_205 === true)) {
        return Ordering["equal"];
      } else if ((__match_205 === false)) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_205);
      }
    })(((a, b) => {
      return (a === b);
    })(__lto_self_953, __lto_other_954)) : __lumo_match_error(__match_204)));
    const __match_201 = ((__match_206[LUMO_TAG] === "less") ? true : ((__match_206[LUMO_TAG] === "equal") ? false : ((__match_206[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_206))));
    if ((__match_201 === true)) {
      const __lto_msg_957 = "Usage: langue <input.langue> [output_dir]";
      const __lto__err_958 = __console_error(__lto_msg_957);
      return __k(__exit_process(1));
    } else if ((__match_201 === false)) {
      const __lto_idx_959 = 1;
      const file = __argv_at_raw(((__lto___lto_self_1217_1230) => {
        const __lto___lto_other_1218_1231 = 1;
        return (__lto___lto_self_1221_1225 + __lto___lto_other_1222_1226);
      })(__lto_idx_959));
      const src = readFileSync(file, "utf8");
      return parse_grammar(__caps, src, (__cps_v_32) => {
        if ((__cps_v_32[LUMO_TAG] === "ok")) {
          const raw_grammar = __cps_v_32.args[0];
          return resolve_grammar(__caps, raw_grammar, (grammar) => {
            if ((grammar[LUMO_TAG] === "mk")) {
              const tokens = grammar.args[0];
              const rules = grammar.args[1];
              const count = list_length_rules__lto_92991de6(rules);
              return generate_syntax_kind__lto_1ba4622a(__caps, grammar, (syntax_kind_code) => {
                return generate_ast__lto_1ba4622a(__caps, grammar, (ast_code) => {
                  return __k(run_generate__lto_35421161(file, count, syntax_kind_code, ast_code));
                });
              });
            } else {
              return __lumo_match_error(grammar);
            }
          });
        } else if ((__cps_v_32[LUMO_TAG] === "err")) {
          const msg = __cps_v_32.args[0];
          const pos = __cps_v_32.args[1];
          const __lto_self_967 = "Parse error at position ";
          return Number.to_string(__caps, pos, (__lto_other_968) => {
            const __lto_self_965 = (__lto___lto_self_1221_1225 + __lto___lto_other_1222_1226);
            const __lto_other_966 = ": ";
            const __lto_self_963 = (__lto_self_953 + __lto_other_954);
            const __lto_msg_961 = (__lto_self_963 + msg);
            const __lto__err_962 = __console_error(__lto_msg_961);
            return __k(__exit_process(1));
          });
        } else {
          return __lumo_match_error(__cps_v_32);
        }
      });
    } else {
      return __lumo_match_error(__match_201);
    }
  });
}

export function run_generate__lto_35421161(file, count, syntax_kind_code, ast_code) {
  const __lto___lto_self_1221_1234 = __argv_length_raw();
  const __lto___lto_other_1222_1235 = 1;
  const __lto_self_975 = (__lto___lto_self_1221_1234 - __lto___lto_other_1222_1235);
  const __lto_other_976 = 3;
  const __match_207 = (__lto_self_975 < __lto_other_976);
  const __match_209 = ((__match_207 === true) ? Ordering["less"] : ((__match_207 === false) ? ((__match_208) => {
    if ((__match_208 === true)) {
      return Ordering["equal"];
    } else if ((__match_208 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_208);
    }
  })(((a, b) => {
    return (a === b);
  })(__lto_self_975, __lto_other_976)) : __lumo_match_error(__match_207)));
  const __match_210 = ((__match_209[LUMO_TAG] === "less") ? true : ((__match_209[LUMO_TAG] === "equal") ? false : ((__match_209[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_209))));
  if ((__match_210 === true)) {
    return write_output__lto_155dcaa4(".", file, count, syntax_kind_code, ast_code);
  } else if ((__match_210 === false)) {
    const __lto_idx_979 = 2;
    const out_dir = __argv_at_raw(((__lto___lto_self_1217_1239) => {
      const __lto___lto_other_1218_1240 = 1;
      return (__lto___lto_self_1221_1234 + __lto___lto_other_1222_1235);
    })(__lto_idx_979));
    return write_output__lto_155dcaa4(out_dir, file, count, syntax_kind_code, ast_code);
  } else {
    return __lumo_match_error(__match_210);
  }
}

export function write_output__lto_155dcaa4(out_dir, file, count, syntax_kind_code, ast_code) {
  const __lto_other_981 = "/syntax_kind.rs";
  const sk_path = (out_dir + __lto_other_981);
  const __lto_other_985 = "/ast.rs";
  const ast_path = (out_dir + __lto_other_985);
  const w1 = writeFileSync(sk_path, syntax_kind_code, "utf8");
  const w2 = writeFileSync(ast_path, ast_code, "utf8");
  const __lto_self_997 = "Parsed ";
  const __lto_other_998 = Number.to_string(count);
  const __lto_self_995 = (__lto_self_997 + __lto_other_998);
  const __lto_other_996 = " rules from ";
  const __lto_self_993 = (__lto_self_995 + __lto_other_996);
  const __lto_msg_992 = (__lto_self_993 + file);
  const p1 = globalThis.console.log(__lto_msg_992);
  const __lto_self_1006 = "Wrote ";
  const __lto_msg_1005 = (__lto_self_1006 + sk_path);
  const p2 = globalThis.console.log(__lto_msg_1005);
  const __lto_self_1011 = "Wrote ";
  const __lto_msg_1010 = (__lto_self_1011 + ast_path);
  return globalThis.console.log(__lto_msg_1010);
}

export function list_length_rules__lto_92991de6(xs) {
  if ((xs[LUMO_TAG] === "nil")) {
    return 0;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const rest = xs.args[1];
    const __lto_self_1015 = 1;
    const __lto_other_1016 = list_length_rules__lto_92991de6(rest);
    return (__lto_self_1015 + __lto_other_1016);
  } else {
    return __lumo_match_error(xs);
  }
}

export function is_whitespace__lto_3890158f(c) {
  const __lto_other_1020 = " ";
  const __match_212 = (c === __lto_other_1020);
  if ((__match_212 === true)) {
    return true;
  } else if ((__match_212 === false)) {
    const __lto_other_1024 = "\n";
    const __match_213 = (c === __lto_other_1020);
    if ((__match_213 === true)) {
      return true;
    } else if ((__match_213 === false)) {
      const __lto_other_1028 = "\t";
      const __match_214 = (c === __lto_other_1020);
      if ((__match_214 === true)) {
        return true;
      } else if ((__match_214 === false)) {
        const __lto_other_1032 = "\r";
        const __match_215 = (c === __lto_other_1020);
        if ((__match_215 === true)) {
          return true;
        } else if ((__match_215 === false)) {
          return false;
        } else {
          return __lumo_match_error(__match_215);
        }
      } else {
        return __lumo_match_error(__match_214);
      }
    } else {
      return __lumo_match_error(__match_213);
    }
  } else {
    return __lumo_match_error(__match_212);
  }
}

export function is_alpha__lto_9309ae26(c) {
  const code = String.char_code_at(c, 0);
  const __lto_other_1036 = 97;
  const __match_216 = (code < __lto_other_1036);
  const __match_218 = ((__match_216 === true) ? Ordering["less"] : ((__match_216 === false) ? ((__match_217) => {
    if ((__match_217 === true)) {
      return Ordering["equal"];
    } else if ((__match_217 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_217);
    }
  })(((a, b) => {
    return (a === b);
  })(code, __lto_other_1036)) : __lumo_match_error(__match_216)));
  const __match_219 = ((__match_218[LUMO_TAG] === "less") ? false : ((__match_218[LUMO_TAG] === "equal") ? true : ((__match_218[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_218))));
  if ((__match_219 === true)) {
    let __match_229;
    const __lto_other_1040 = 122;
    const __match_227 = (code < __lto_other_1036);
    if ((__match_227 === true)) {
      __match_229 = Ordering["less"];
    } else if ((__match_227 === false)) {
      const __match_228 = (code === __lto_other_1036);
      if ((__match_228 === true)) {
        __match_229 = Ordering["equal"];
      } else if ((__match_228 === false)) {
        __match_229 = Ordering["greater"];
      } else {
        __match_229 = __lumo_match_error(__match_228);
      }
    } else {
      __match_229 = __lumo_match_error(__match_227);
    }
    if ((__match_229[LUMO_TAG] === "less")) {
      return true;
    } else if ((__match_229[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return ((__match_229[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_229));
    }
  } else if ((__match_219 === false)) {
    let __match_223;
    let __match_222;
    const __lto_other_1044 = 65;
    const __match_220 = (code < __lto_other_1036);
    if ((__match_220 === true)) {
      __match_222 = Ordering["less"];
    } else if ((__match_220 === false)) {
      const __match_221 = (code === __lto_other_1036);
      if ((__match_221 === true)) {
        __match_222 = Ordering["equal"];
      } else if ((__match_221 === false)) {
        __match_222 = Ordering["greater"];
      } else {
        __match_222 = __lumo_match_error(__match_221);
      }
    } else {
      __match_222 = __lumo_match_error(__match_220);
    }
    if ((__match_222[LUMO_TAG] === "less")) {
      __match_223 = false;
    } else if ((__match_222[LUMO_TAG] === "equal")) {
      __match_223 = true;
    } else {
      __match_223 = ((__match_222[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_222));
    }
    if ((__match_223 === true)) {
      let __match_226;
      const __lto_other_1048 = 90;
      const __match_224 = (code < __lto_other_1036);
      if ((__match_224 === true)) {
        __match_226 = Ordering["less"];
      } else if ((__match_224 === false)) {
        const __match_225 = (code === __lto_other_1036);
        if ((__match_225 === true)) {
          __match_226 = Ordering["equal"];
        } else if ((__match_225 === false)) {
          __match_226 = Ordering["greater"];
        } else {
          __match_226 = __lumo_match_error(__match_225);
        }
      } else {
        __match_226 = __lumo_match_error(__match_224);
      }
      if ((__match_226[LUMO_TAG] === "less")) {
        return true;
      } else if ((__match_226[LUMO_TAG] === "equal")) {
        return true;
      } else {
        return ((__match_226[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_226));
      }
    } else if ((__match_223 === false)) {
      return false;
    } else {
      return __lumo_match_error(__match_223);
    }
  } else {
    return __lumo_match_error(__match_219);
  }
}

export function is_ident_continue__lto_3890158f(c) {
  const __match_230 = is_alpha__lto_9309ae26(c);
  if ((__match_230 === true)) {
    return true;
  } else if ((__match_230 === false)) {
    const __lto_other_1052 = "_";
    const __match_231 = (c === __lto_other_1052);
    if ((__match_231 === true)) {
      return true;
    } else if ((__match_231 === false)) {
      return false;
    } else {
      return __lumo_match_error(__match_231);
    }
  } else {
    return __lumo_match_error(__match_230);
  }
}

export function state_eof__lto_9309ae26(st) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    const __lto_other_1056 = String.len(src);
    const __match_233 = (pos < __lto_other_1056);
    const __match_235 = ((__match_233 === true) ? Ordering["less"] : ((__match_233 === false) ? ((__match_234) => {
      if ((__match_234 === true)) {
        return Ordering["equal"];
      } else if ((__match_234 === false)) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_234);
      }
    })(((a, b) => {
      return (a === b);
    })(pos, __lto_other_1056)) : __lumo_match_error(__match_233)));
    if ((__match_235[LUMO_TAG] === "less")) {
      return false;
    } else if ((__match_235[LUMO_TAG] === "equal")) {
      return true;
    } else {
      return ((__match_235[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_235));
    }
  } else {
    return __lumo_match_error(st);
  }
}

export function state_peek__lto_9309ae26(st) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    const __lto_other_1060 = String.len(src);
    const __match_237 = (pos < __lto_other_1060);
    const __match_239 = ((__match_237 === true) ? Ordering["less"] : ((__match_237 === false) ? ((__match_238) => {
      if ((__match_238 === true)) {
        return Ordering["equal"];
      } else if ((__match_238 === false)) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_238);
      }
    })(((a, b) => {
      return (a === b);
    })(pos, __lto_other_1060)) : __lumo_match_error(__match_237)));
    const __match_240 = ((__match_239[LUMO_TAG] === "less") ? true : ((__match_239[LUMO_TAG] === "equal") ? false : ((__match_239[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_239))));
    if ((__match_240 === true)) {
      return String.char_at(src, pos);
    } else if ((__match_240 === false)) {
      return "";
    } else {
      return __lumo_match_error(__match_240);
    }
  } else {
    return __lumo_match_error(st);
  }
}

export function state_advance__lto_92991de6(st, n) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    return ParseState["mk"](src, ((__lto_self_1063) => {
      return (__lto_self_1063 + n);
    })(pos));
  } else {
    return __lumo_match_error(st);
  }
}

export function skip_ws__lto_1bb67705(st) {
  const __match_242 = state_eof__lto_9309ae26(st);
  if ((__match_242 === true)) {
    return st;
  } else if ((__match_242 === false)) {
    const c = state_peek__lto_9309ae26(st);
    const __match_243 = is_whitespace__lto_3890158f(c);
    if ((__match_243 === true)) {
      return skip_ws__lto_1bb67705(state_advance__lto_92991de6(st, 1));
    } else if ((__match_243 === false)) {
      const __lto_other_1068 = "/";
      const __match_244 = (c === __lto_other_1068);
      if ((__match_244 === true)) {
        const __lto_self_1071 = state_pos(st);
        const __lto_other_1072 = 1;
        const next_pos = (c + __lto_other_1068);
        const __lto_other_1076 = String.len(state_src(st));
        const __match_245 = (next_pos < __lto_other_1076);
        const __match_247 = ((__match_245 === true) ? Ordering["less"] : ((__match_245 === false) ? ((__match_246) => {
          if ((__match_246 === true)) {
            return Ordering["equal"];
          } else if ((__match_246 === false)) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_246);
          }
        })(((a, b) => {
          return (a === b);
        })(next_pos, __lto_other_1076)) : __lumo_match_error(__match_245)));
        const __match_248 = ((__match_247[LUMO_TAG] === "less") ? true : ((__match_247[LUMO_TAG] === "equal") ? false : ((__match_247[LUMO_TAG] === "greater") ? false : __lumo_match_error(__match_247))));
        if ((__match_248 === true)) {
          const __lto_self_1079 = String.char_at(state_src(st), next_pos);
          const __lto_other_1080 = "/";
          const __match_249 = (c === __lto_other_1068);
          if ((__match_249 === true)) {
            return skip_ws__lto_1bb67705(skip_line__lto_3890158f(state_advance__lto_92991de6(st, 2)));
          } else if ((__match_249 === false)) {
            return st;
          } else {
            return __lumo_match_error(__match_249);
          }
        } else if ((__match_248 === false)) {
          return st;
        } else {
          return __lumo_match_error(__match_248);
        }
      } else if ((__match_244 === false)) {
        return st;
      } else {
        return __lumo_match_error(__match_244);
      }
    } else {
      return __lumo_match_error(__match_243);
    }
  } else {
    return __lumo_match_error(__match_242);
  }
}

export function skip_line__lto_3890158f(st) {
  const __match_250 = state_eof__lto_9309ae26(st);
  if ((__match_250 === true)) {
    return st;
  } else if ((__match_250 === false)) {
    const __lto_self_1083 = state_peek__lto_9309ae26(st);
    const __lto_other_1084 = "\n";
    const __match_251 = (__lto_self_1083 === __lto_other_1084);
    if ((__match_251 === true)) {
      return state_advance__lto_92991de6(st, 1);
    } else if ((__match_251 === false)) {
      return skip_line__lto_3890158f(state_advance__lto_92991de6(st, 1));
    } else {
      return __lumo_match_error(__match_251);
    }
  } else {
    return __lumo_match_error(__match_250);
  }
}

export function parse_ident__lto_1ba4622a(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const __match_252 = state_eof__lto_9309ae26(st2);
    if ((__match_252 === true)) {
      return __k(ParseResult["err"]("expected identifier, got EOF", state_pos(st2)));
    } else if ((__match_252 === false)) {
      return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_34) => {
        if ((__cps_v_34 === true)) {
          const start = state_pos(st2);
          return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (end_st) => {
            const end_pos = state_pos(end_st);
            return String.slice(__caps, state_src(st2), start, end_pos, (__cps_v_33) => {
              return __k(ParseResult["ok"](__cps_v_33, end_st));
            });
          });
        } else if ((__cps_v_34 === false)) {
          return __k(ParseResult["err"](((__lto_self_1087) => {
            const __lto_other_1088 = "'";
            return (__lto_self_1087 + __lto_other_1088);
          })(((__lto_self_1089) => {
            const __lto_other_1090 = state_peek__lto_9309ae26(st2);
            return (__lto_self_1089 + __lto_other_1090);
          })("expected identifier, got '")), state_pos(st2)));
        } else {
          return __lumo_match_error(__cps_v_34);
        }
      });
    } else {
      return __lumo_match_error(__match_252);
    }
  });
}

export function expect__lto_f3280589(st, expected) {
  const st2 = skip_ws__lto_1bb67705(st);
  const len = String.len(expected);
  const src = state_src(st2);
  const pos = state_pos(st2);
  const __lto_self_1095 = String.len(src);
  const remaining = (__lto_self_1095 - pos);
  const __match_254 = (remaining < len);
  const __match_256 = ((__match_254 === true) ? Ordering["less"] : ((__match_254 === false) ? ((__match_255) => {
    if ((__match_255 === true)) {
      return Ordering["equal"];
    } else if ((__match_255 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_255);
    }
  })(((a, b) => {
    return (a === b);
  })(remaining, len)) : __lumo_match_error(__match_254)));
  const __match_257 = ((__match_256[LUMO_TAG] === "less") ? false : ((__match_256[LUMO_TAG] === "equal") ? true : ((__match_256[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_256))));
  if ((__match_257 === true)) {
    const slice = String.slice(src, pos, ((__lto_self_1103) => {
      return (__lto_self_1095 + pos);
    })(pos));
    const __match_258 = (__lto_self_1095 === pos);
    if ((__match_258 === true)) {
      return ParseResult["ok"](expected, state_advance__lto_92991de6(st2, len));
    } else if ((__match_258 === false)) {
      return ParseResult["err"](((__lto_self_1111) => {
        const __lto_other_1112 = "'";
        return (__lto_self_1095 + pos);
      })(((__lto_self_1113) => {
        return (__lto_self_1095 + pos);
      })(((__lto_self_1115) => {
        const __lto_other_1116 = "', got '";
        return (__lto_self_1095 + pos);
      })(((__lto_self_1117) => {
        return (__lto_self_1095 + pos);
      })("expected '")))), pos);
    } else {
      return __lumo_match_error(__match_258);
    }
  } else if ((__match_257 === false)) {
    return ParseResult["err"](((__lto_self_1127) => {
      const __lto_other_1128 = "'";
      return (__lto_self_1095 + pos);
    })(((__lto_self_1129) => {
      return (__lto_self_1095 + pos);
    })("expected '")), pos);
  } else {
    return __lumo_match_error(__match_257);
  }
}

export function parse_quoted__lto_38e07bea(st) {
  const st2 = skip_ws__lto_1bb67705(st);
  const __lto_self_1135 = state_peek__lto_9309ae26(st2);
  const __lto_other_1136 = "'";
  const __match_259 = (__lto_self_1135 === __lto_other_1136);
  if ((__match_259 === true)) {
    const __lto_self_1139 = state_pos(st2);
    const __lto_other_1140 = 1;
    const start = (__lto_self_1135 + __lto_other_1136);
    const end_st = scan_until_quote__lto_3890158f(state_advance__lto_92991de6(st2, 1));
    const end_pos = state_pos(end_st);
    const content = String.slice(state_src(st2), start, end_pos);
    return ParseResult["ok"](content, state_advance__lto_92991de6(end_st, 1));
  } else if ((__match_259 === false)) {
    return ParseResult["err"]("expected quoted literal", state_pos(st2));
  } else {
    return __lumo_match_error(__match_259);
  }
}

export function scan_until_quote__lto_3890158f(st) {
  const __match_260 = state_eof__lto_9309ae26(st);
  if ((__match_260 === true)) {
    return st;
  } else if ((__match_260 === false)) {
    const __lto_self_1143 = state_peek__lto_9309ae26(st);
    const __lto_other_1144 = "'";
    const __match_261 = (__lto_self_1143 === __lto_other_1144);
    if ((__match_261 === true)) {
      return st;
    } else if ((__match_261 === false)) {
      return scan_until_quote__lto_3890158f(state_advance__lto_92991de6(st, 1));
    } else {
      return __lumo_match_error(__match_261);
    }
  } else {
    return __lumo_match_error(__match_260);
  }
}

export function peek_is_rule_start__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_35) => {
      if ((__cps_v_35 === true)) {
        return scan_ident_rest(__caps, state_advance__lto_92991de6(st2, 1), (st3) => {
          const st4 = skip_ws__lto_1bb67705(st3);
          const __lto_self_1147 = state_peek__lto_9309ae26(st4);
          const __lto_other_1148 = "=";
          return __k(((a, b) => {
            return (a === b);
          })(__lto_self_1147, __lto_other_1148));
        });
      } else if ((__cps_v_35 === false)) {
        return __k(false);
      } else {
        return __lumo_match_error(__cps_v_35);
      }
    });
  });
}

export function has_alpha__lto_090deca7(s, i) {
  const __lto_other_1152 = String.len(s);
  const __match_263 = (i < __lto_other_1152);
  const __match_265 = ((__match_263 === true) ? Ordering["less"] : ((__match_263 === false) ? ((__match_264) => {
    if ((__match_264 === true)) {
      return Ordering["equal"];
    } else if ((__match_264 === false)) {
      return Ordering["greater"];
    } else {
      return __lumo_match_error(__match_264);
    }
  })(((a, b) => {
    return (a === b);
  })(i, __lto_other_1152)) : __lumo_match_error(__match_263)));
  const __match_266 = ((__match_265[LUMO_TAG] === "less") ? false : ((__match_265[LUMO_TAG] === "equal") ? true : ((__match_265[LUMO_TAG] === "greater") ? true : __lumo_match_error(__match_265))));
  if ((__match_266 === true)) {
    return false;
  } else if ((__match_266 === false)) {
    const __match_267 = is_alpha__lto_9309ae26(String.char_at(s, i));
    if ((__match_267 === true)) {
      return true;
    } else if ((__match_267 === false)) {
      return has_alpha__lto_090deca7(s, ((__lto_self_1155) => {
        const __lto_other_1156 = 1;
        return (i + __lto_other_1152);
      })(i));
    } else {
      return __lumo_match_error(__match_267);
    }
  } else {
    return __lumo_match_error(__match_266);
  }
}

export function parse_grammar_items__lto_3890158f(__caps, st, tokens, rules, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const __match_268 = state_eof__lto_9309ae26(st2);
    if ((__match_268 === true)) {
      return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_rule(rules)), st2));
    } else if ((__match_268 === false)) {
      const __lto_self_1159 = state_peek__lto_9309ae26(st2);
      const __lto_other_1160 = "@";
      const __match_269 = (__lto_self_1159 === __lto_other_1160);
      if ((__match_269 === true)) {
        return parse_token_def(__caps, st2, (__cps_v_37) => {
          if ((__cps_v_37[LUMO_TAG] === "ok")) {
            const new_tokens = __cps_v_37.args[0];
            const st3 = __cps_v_37.args[1];
            return parse_grammar_items__lto_3890158f(__caps, st3, list_concat_string(new_tokens, tokens), rules, __k);
          } else if ((__cps_v_37[LUMO_TAG] === "err")) {
            const msg = __cps_v_37.args[0];
            const pos = __cps_v_37.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_37);
          }
        });
      } else if ((__match_269 === false)) {
        return parse_rule(__caps, st2, (__cps_v_36) => {
          if ((__cps_v_36[LUMO_TAG] === "ok")) {
            const rule = __cps_v_36.args[0];
            const st3 = __cps_v_36.args[1];
            return parse_grammar_items__lto_3890158f(__caps, st3, tokens, List["cons"](rule, rules), __k);
          } else if ((__cps_v_36[LUMO_TAG] === "err")) {
            const msg = __cps_v_36.args[0];
            const pos = __cps_v_36.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_36);
          }
        });
      } else {
        return __lumo_match_error(__match_269);
      }
    } else {
      return __lumo_match_error(__match_268);
    }
  });
}

export function parse_token_names__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const __match_272 = state_eof__lto_9309ae26(st2);
    if ((__match_272 === true)) {
      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
    } else if ((__match_272 === false)) {
      return peek_is_rule_start__lto_3890158f(__caps, st2, (__cps_v_40) => {
        if ((__cps_v_40 === true)) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((__cps_v_40 === false)) {
          const __lto_self_1163 = state_peek__lto_9309ae26(st2);
          const __lto_other_1164 = "@";
          const __match_274 = (__lto_self_1163 === __lto_other_1164);
          if ((__match_274 === true)) {
            return __k(ParseResult["ok"](list_reverse_string(acc), st2));
          } else if ((__match_274 === false)) {
            return is_ident_start(__caps, state_peek__lto_9309ae26(st2), (__cps_v_39) => {
              if ((__cps_v_39 === true)) {
                return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_38) => {
                  if ((__cps_v_38[LUMO_TAG] === "ok")) {
                    const name = __cps_v_38.args[0];
                    const st3 = __cps_v_38.args[1];
                    return parse_token_names__lto_3890158f(__caps, st3, List["cons"](name, acc), __k);
                  } else if ((__cps_v_38[LUMO_TAG] === "err")) {
                    const msg = __cps_v_38.args[0];
                    const pos = __cps_v_38.args[1];
                    return __k(ParseResult["err"](msg, pos));
                  } else {
                    return __lumo_match_error(__cps_v_38);
                  }
                });
              } else if ((__cps_v_39 === false)) {
                return __k(ParseResult["ok"](list_reverse_string(acc), st2));
              } else {
                return __lumo_match_error(__cps_v_39);
              }
            });
          } else {
            return __lumo_match_error(__match_274);
          }
        } else {
          return __lumo_match_error(__cps_v_40);
        }
      });
    } else {
      return __lumo_match_error(__match_272);
    }
  });
}

export function parse_rule_body__lto_3890158f(__caps, st, rule_name, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_char(__caps, st2, (__lto_self_1167) => {
      const __lto_other_1168 = "|";
      const __cps_v_41 = (__lto_self_1167 === __lto_other_1168);
      if ((__cps_v_41 === true)) {
        return parse_alternatives(__caps, st2, __k);
      } else if ((__cps_v_41 === false)) {
        return parse_sequence(__caps, st2, __k);
      } else {
        return __lumo_match_error(__cps_v_41);
      }
    });
  });
}

export function parse_alt_items__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    return peek_char(__caps, st2, (__lto_self_1171) => {
      const __lto_other_1172 = "|";
      const __cps_v_43 = (__lto_self_1171 === __lto_other_1172);
      if ((__cps_v_43 === true)) {
        const st3 = state_advance__lto_92991de6(skip_ws__lto_1bb67705(st2), 1);
        const st4 = skip_ws__lto_1bb67705(st3);
        const __lto_self_1175 = state_peek__lto_9309ae26(st4);
        const __lto_other_1176 = "'";
        const __match_279 = (__lto_self_1171 === __lto_other_1172);
        if ((__match_279 === true)) {
          const __match_281 = parse_quoted__lto_38e07bea(st4);
          if ((__match_281[LUMO_TAG] === "ok")) {
            const lit = __match_281.args[0];
            const st5 = __match_281.args[1];
            return parse_alt_items__lto_3890158f(__caps, st5, List["cons"](Alternative["mk"](lit), acc), __k);
          } else if ((__match_281[LUMO_TAG] === "err")) {
            const msg = __match_281.args[0];
            const pos = __match_281.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__match_281);
          }
        } else if ((__match_279 === false)) {
          return parse_ident__lto_1ba4622a(__caps, st3, (__cps_v_42) => {
            if ((__cps_v_42[LUMO_TAG] === "ok")) {
              const name = __cps_v_42.args[0];
              const st5 = __cps_v_42.args[1];
              return parse_alt_items__lto_3890158f(__caps, st5, List["cons"](Alternative["mk"](name), acc), __k);
            } else if ((__cps_v_42[LUMO_TAG] === "err")) {
              const msg = __cps_v_42.args[0];
              const pos = __cps_v_42.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_42);
            }
          });
        } else {
          return __lumo_match_error(__match_279);
        }
      } else if ((__cps_v_43 === false)) {
        return __k(ParseResult["ok"](RuleBody["alternatives"](list_reverse_alt(acc)), st2));
      } else {
        return __lumo_match_error(__cps_v_43);
      }
    });
  });
}

export function is_seq_terminator__lto_3890158f(__caps, st, __k) {
  return peek_char(__caps, st, (c) => {
    const __lto_other_1180 = ")";
    const __match_282 = (c === __lto_other_1180);
    if ((__match_282 === true)) {
      return __k(true);
    } else if ((__match_282 === false)) {
      return peek_is_rule_start__lto_3890158f(__caps, st, (__cps_v_44) => {
        if ((__cps_v_44 === true)) {
          return __k(true);
        } else if ((__cps_v_44 === false)) {
          const __lto_other_1184 = "@";
          const __match_284 = (c === __lto_other_1180);
          if ((__match_284 === true)) {
            return __k(true);
          } else if ((__match_284 === false)) {
            return __k(false);
          } else {
            return __lumo_match_error(__match_284);
          }
        } else {
          return __lumo_match_error(__cps_v_44);
        }
      });
    } else {
      return __lumo_match_error(__match_282);
    }
  });
}

export function apply_postfix_elem__lto_3890158f(elem, st) {
  const __match_285 = state_eof__lto_9309ae26(st);
  if ((__match_285 === true)) {
    return ParseResult["ok"](elem, st);
  } else if ((__match_285 === false)) {
    const __lto_self_1187 = state_peek__lto_9309ae26(st);
    const __lto_other_1188 = "?";
    const __match_286 = (__lto_self_1187 === __lto_other_1188);
    if ((__match_286 === true)) {
      return apply_postfix_elem__lto_3890158f(Element["optional"](elem), state_advance__lto_92991de6(st, 1));
    } else if ((__match_286 === false)) {
      const __lto_self_1191 = state_peek__lto_9309ae26(st);
      const __lto_other_1192 = "*";
      const __match_287 = (__lto_self_1187 === __lto_other_1188);
      if ((__match_287 === true)) {
        return apply_postfix_elem__lto_3890158f(Element["repeated"](elem), state_advance__lto_92991de6(st, 1));
      } else if ((__match_287 === false)) {
        return ParseResult["ok"](elem, st);
      } else {
        return __lumo_match_error(__match_287);
      }
    } else {
      return __lumo_match_error(__match_286);
    }
  } else {
    return __lumo_match_error(__match_285);
  }
}

export function parse_atom__lto_3890158f(__caps, st, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const __lto_self_1195 = state_peek__lto_9309ae26(st2);
    const __lto_other_1196 = "'";
    const __match_288 = (__lto_self_1195 === __lto_other_1196);
    if ((__match_288 === true)) {
      const __match_295 = parse_quoted__lto_38e07bea(st2);
      if ((__match_295[LUMO_TAG] === "ok")) {
        const lit = __match_295.args[0];
        const st3 = __match_295.args[1];
        return classify_literal(__caps, lit, (__cps_v_49) => {
          const __cps_v_48 = Element["token"](__cps_v_49);
          return __k(ParseResult["ok"](__cps_v_48, st3));
        });
      } else if ((__match_295[LUMO_TAG] === "err")) {
        const msg = __match_295.args[0];
        const pos = __match_295.args[1];
        return __k(ParseResult["err"](msg, pos));
      } else {
        return __lumo_match_error(__match_295);
      }
    } else if ((__match_288 === false)) {
      const __lto_self_1199 = state_peek__lto_9309ae26(st2);
      const __lto_other_1200 = "(";
      const __match_289 = (__lto_self_1195 === __lto_other_1196);
      if ((__match_289 === true)) {
        const st3 = state_advance__lto_92991de6(st2, 1);
        return parse_group_elements__lto_3890158f(__caps, st3, List["nil"], (__cps_v_47) => {
          if ((__cps_v_47[LUMO_TAG] === "ok")) {
            const elems = __cps_v_47.args[0];
            const st4 = __cps_v_47.args[1];
            const __match_294 = expect__lto_f3280589(st4, ")");
            if ((__match_294[LUMO_TAG] === "ok")) {
              const st5 = __match_294.args[1];
              return __k(ParseResult["ok"](Element["group"](elems), st5));
            } else if ((__match_294[LUMO_TAG] === "err")) {
              const msg = __match_294.args[0];
              const pos = __match_294.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__match_294);
            }
          } else if ((__cps_v_47[LUMO_TAG] === "err")) {
            const msg = __cps_v_47.args[0];
            const pos = __cps_v_47.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_47);
          }
        });
      } else if ((__match_289 === false)) {
        return parse_ident__lto_1ba4622a(__caps, st2, (__cps_v_46) => {
          if ((__cps_v_46[LUMO_TAG] === "err")) {
            const msg = __cps_v_46.args[0];
            const pos = __cps_v_46.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else if ((__cps_v_46[LUMO_TAG] === "ok")) {
            const name = __cps_v_46.args[0];
            const st3 = __cps_v_46.args[1];
            const __lto_self_1203 = state_peek__lto_9309ae26(st3);
            const __lto_other_1204 = ":";
            const __match_291 = (__lto_self_1195 === __lto_other_1196);
            if ((__match_291 === true)) {
              const st4 = state_advance__lto_92991de6(st3, 1);
              return parse_element(__caps, st4, (__cps_v_45) => {
                if ((__cps_v_45[LUMO_TAG] === "ok")) {
                  const inner = __cps_v_45.args[0];
                  const st5 = __cps_v_45.args[1];
                  return __k(ParseResult["ok"](Element["labeled"](name, inner), st5));
                } else if ((__cps_v_45[LUMO_TAG] === "err")) {
                  const msg = __cps_v_45.args[0];
                  const pos = __cps_v_45.args[1];
                  return __k(ParseResult["err"](msg, pos));
                } else {
                  return __lumo_match_error(__cps_v_45);
                }
              });
            } else if ((__match_291 === false)) {
              return __k(ParseResult["ok"](Element["node"](NodeRef["mk"](name)), st3));
            } else {
              return __lumo_match_error(__match_291);
            }
          } else {
            return __lumo_match_error(__cps_v_46);
          }
        });
      } else {
        return __lumo_match_error(__match_289);
      }
    } else {
      return __lumo_match_error(__match_288);
    }
  });
}

export function parse_group_elements__lto_3890158f(__caps, st, acc, __k) {
  return __thunk(() => {
    const st2 = skip_ws__lto_1bb67705(st);
    const __lto_self_1207 = state_peek__lto_9309ae26(st2);
    const __lto_other_1208 = ")";
    const __match_296 = (__lto_self_1207 === __lto_other_1208);
    if ((__match_296 === true)) {
      return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
    } else if ((__match_296 === false)) {
      return parse_element(__caps, st2, (__cps_v_50) => {
        if ((__cps_v_50[LUMO_TAG] === "ok")) {
          const elem = __cps_v_50.args[0];
          const st3 = __cps_v_50.args[1];
          return parse_group_elements__lto_3890158f(__caps, st3, List["cons"](elem, acc), __k);
        } else if ((__cps_v_50[LUMO_TAG] === "err")) {
          const msg = __cps_v_50.args[0];
          const pos = __cps_v_50.args[1];
          return __k(ParseResult["err"](msg, pos));
        } else {
          return __lumo_match_error(__cps_v_50);
        }
      });
    } else {
      return __lumo_match_error(__match_296);
    }
  });
}

export function list_contains_string__lto_3890158f(xs, target) {
  if ((xs[LUMO_TAG] === "nil")) {
    return false;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const x = xs.args[0];
    const rest = xs.args[1];
    const __match_299 = (x === target);
    if ((__match_299 === true)) {
      return true;
    } else if ((__match_299 === false)) {
      return list_contains_string__lto_3890158f(rest, target);
    } else {
      return __lumo_match_error(__match_299);
    }
  } else {
    return __lumo_match_error(xs);
  }
}

main();
