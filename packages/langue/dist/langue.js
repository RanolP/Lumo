const LUMO_TAG = Symbol.for("Lumo/tag");
const __lumo_match_error = (value) => { throw new Error("non-exhaustive match: " + JSON.stringify(value)); };
const __thunk = (fn) => { fn.__t = 1; return fn; };
const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };
const __identity = (__v) => __v;

import { readFileSync as __lumo_readFileSync, writeFileSync as __lumo_writeFileSync } from "node:fs";



function __main_cps(__caps, __k) {
  return run(__caps, __k);
}

export function main() {
  return __trampoline(__main_cps({ IO_IO: IO, StrOps_StrOps: StrOps, Add_String: __impl_String_Add, Sub_Number: __impl_Number_Sub, NumOps_NumOps: NumOps, Add_Number: __impl_Number_Add, PartialEq_String: __impl_String_PartialEq, FS_FS: FS, Process_Process: Process, PartialOrd_Number: __impl_Number_PartialOrd }, __identity));
}

export function run(__caps, __k) {
  return __thunk(() => {
    return __caps.Process_Process.args_count(__caps, (__cps_v_7) => {
      return __caps.PartialOrd_Number.cmp(__caps, __cps_v_7, 2, (__cps_v_6) => {
        const __k_3 = (__cps_v_5) => {
          if ((__cps_v_5[LUMO_TAG] === "true")) {
            return __caps.Process_Process.panic_with(__caps, "Usage: langue <input.langue> [output_dir]", __k);
          } else if ((__cps_v_5[LUMO_TAG] === "false")) {
            return __caps.Process_Process.arg_at(__caps, 1, (file) => {
              return __caps.FS_FS.read_file(__caps, file, (src) => {
                return parse_grammar(__caps, src, (__cps_v_4) => {
                  if ((__cps_v_4[LUMO_TAG] === "ok")) {
                    const raw_grammar = __cps_v_4.args[0];
                    return resolve_grammar(__caps, raw_grammar, (grammar) => {
                      if ((grammar[LUMO_TAG] === "mk")) {
                        const tokens = grammar.args[0];
                        const rules = grammar.args[1];
                        return list_length_rules(__caps, rules, (count) => {
                          return generate_syntax_kind(__caps, grammar, (syntax_kind_code) => {
                            return generate_ast(__caps, grammar, (ast_code) => {
                              return run_generate(__caps, file, count, syntax_kind_code, ast_code, __k);
                            });
                          });
                        });
                      } else {
                        return __lumo_match_error(grammar);
                      }
                    });
                  } else if ((__cps_v_4[LUMO_TAG] === "err")) {
                    const msg = __cps_v_4.args[0];
                    const pos = __cps_v_4.args[1];
                    return Number.to_string(__caps, pos, (__cps_v_3) => {
                      return __caps.Add_String.add(__caps, "Parse error at position ", __cps_v_3, (__cps_v_2) => {
                        return __caps.Add_String.add(__caps, __cps_v_2, ": ", (__cps_v_1) => {
                          return __caps.Add_String.add(__caps, __cps_v_1, msg, (__cps_v_0) => {
                            return __caps.Process_Process.panic_with(__caps, __cps_v_0, __k);
                          });
                        });
                      });
                    });
                  } else {
                    return __lumo_match_error(__cps_v_4);
                  }
                });
              });
            });
          } else {
            return __lumo_match_error(__cps_v_5);
          }
        };
        if ((__cps_v_6[LUMO_TAG] === "less")) {
          return __k_3(Bool["true"]);
        } else if ((__cps_v_6[LUMO_TAG] === "equal")) {
          return __k_3(Bool["false"]);
        } else {
          return ((__cps_v_6[LUMO_TAG] === "greater") ? __k_3(Bool["false"]) : __lumo_match_error(__cps_v_6));
        }
      });
    });
  });
}

export function run_generate(__caps, file, count, syntax_kind_code, ast_code, __k) {
  return __thunk(() => {
    return __caps.Process_Process.args_count(__caps, (__cps_v_10) => {
      return __caps.PartialOrd_Number.cmp(__caps, __cps_v_10, 3, (__cps_v_9) => {
        const __k_5 = (__cps_v_8) => {
          if ((__cps_v_8[LUMO_TAG] === "true")) {
            return write_output(__caps, ".", file, count, syntax_kind_code, ast_code, __k);
          } else if ((__cps_v_8[LUMO_TAG] === "false")) {
            return __caps.Process_Process.arg_at(__caps, 2, (out_dir) => {
              return write_output(__caps, out_dir, file, count, syntax_kind_code, ast_code, __k);
            });
          } else {
            return __lumo_match_error(__cps_v_8);
          }
        };
        if ((__cps_v_9[LUMO_TAG] === "less")) {
          return __k_5(Bool["true"]);
        } else if ((__cps_v_9[LUMO_TAG] === "equal")) {
          return __k_5(Bool["false"]);
        } else {
          return ((__cps_v_9[LUMO_TAG] === "greater") ? __k_5(Bool["false"]) : __lumo_match_error(__cps_v_9));
        }
      });
    });
  });
}

export function write_output(__caps, out_dir, file, count, syntax_kind_code, ast_code, __k) {
  return __thunk(() => {
    return __caps.Add_String.add(__caps, out_dir, "/syntax_kind.rs", (sk_path) => {
      return __caps.Add_String.add(__caps, out_dir, "/ast.rs", (ast_path) => {
        return __caps.FS_FS.write_file(__caps, sk_path, syntax_kind_code, (w1) => {
          return __caps.FS_FS.write_file(__caps, ast_path, ast_code, (w2) => {
            return Number.to_string(__caps, count, (__cps_v_16) => {
              return __caps.Add_String.add(__caps, "Parsed ", __cps_v_16, (__cps_v_15) => {
                return __caps.Add_String.add(__caps, __cps_v_15, " rules from ", (__cps_v_14) => {
                  return __caps.Add_String.add(__caps, __cps_v_14, file, (__cps_v_13) => {
                    return __caps.IO_IO.println(__caps, __cps_v_13, (p1) => {
                      return __caps.Add_String.add(__caps, "Wrote ", sk_path, (__cps_v_12) => {
                        return __caps.IO_IO.println(__caps, __cps_v_12, (p2) => {
                          return __caps.Add_String.add(__caps, "Wrote ", ast_path, (__cps_v_11) => {
                            return __caps.IO_IO.println(__caps, __cps_v_11, __k);
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    });
  });
}

export function list_length_rules(__caps, xs, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(0);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const rest = xs.args[1];
      return list_length_rules(__caps, rest, (__cps_v_17) => {
        return __caps.Add_Number.add(__caps, 1, __cps_v_17, __k);
      });
    } else {
      return __lumo_match_error(xs);
    }
  });
}


export const List = { "nil": { [LUMO_TAG]: "nil" }, "cons": (arg0, arg1) => {
  return { [LUMO_TAG]: "cons", args: [arg0, arg1] };
} };

export function list_is_empty(xs) {
  if ((xs[LUMO_TAG] === "nil")) {
    return Bool["true"];
  } else if ((xs[LUMO_TAG] === "cons")) {
    return Bool["false"];
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_reverse_acc(xs, acc) {
  if ((xs[LUMO_TAG] === "nil")) {
    return acc;
  } else if ((xs[LUMO_TAG] === "cons")) {
    const h = xs.args[0];
    const t = xs.args[1];
    return list_reverse_acc(t, List["cons"](h, acc));
  } else {
    return __lumo_match_error(xs);
  }
}

export function list_reverse(xs) {
  return list_reverse_acc(xs, List["nil"]);
}

export function list_length(__caps, xs, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(0);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const t = xs.args[1];
      return list_length(__caps, t, (__cps_v_18) => {
        return __caps.NumOps_NumOps.add(__caps, 1, __cps_v_18, __k);
      });
    } else {
      return __lumo_match_error(xs);
    }
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

export function is_whitespace(__caps, c, __k) {
  return __thunk(() => {
    return __caps.PartialEq_String.eq(__caps, c, " ", (__cps_v_22) => {
      if ((__cps_v_22[LUMO_TAG] === "true")) {
        return __k(Bool["true"]);
      } else if ((__cps_v_22[LUMO_TAG] === "false")) {
        return __caps.PartialEq_String.eq(__caps, c, "\n", (__cps_v_21) => {
          if ((__cps_v_21[LUMO_TAG] === "true")) {
            return __k(Bool["true"]);
          } else if ((__cps_v_21[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, c, "\t", (__cps_v_20) => {
              if ((__cps_v_20[LUMO_TAG] === "true")) {
                return __k(Bool["true"]);
              } else if ((__cps_v_20[LUMO_TAG] === "false")) {
                return __caps.PartialEq_String.eq(__caps, c, "\r", (__cps_v_19) => {
                  if ((__cps_v_19[LUMO_TAG] === "true")) {
                    return __k(Bool["true"]);
                  } else if ((__cps_v_19[LUMO_TAG] === "false")) {
                    return __k(Bool["false"]);
                  } else {
                    return __lumo_match_error(__cps_v_19);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_20);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_21);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_22);
      }
    });
  });
}

export function is_alpha(__caps, c, __k) {
  return String.char_code_at(__caps, c, 0, (code) => {
    return __caps.PartialOrd_Number.cmp(__caps, code, 97, (__cps_v_28) => {
      const __k_17 = (__cps_v_27) => {
        if ((__cps_v_27[LUMO_TAG] === "true")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 122, (__cps_v_26) => {
            if ((__cps_v_26[LUMO_TAG] === "less")) {
              return __k(Bool["true"]);
            } else if ((__cps_v_26[LUMO_TAG] === "equal")) {
              return __k(Bool["true"]);
            } else {
              return ((__cps_v_26[LUMO_TAG] === "greater") ? __k(Bool["false"]) : __lumo_match_error(__cps_v_26));
            }
          });
        } else if ((__cps_v_27[LUMO_TAG] === "false")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_25) => {
            const __k_15 = (__cps_v_24) => {
              if ((__cps_v_24[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_23) => {
                  if ((__cps_v_23[LUMO_TAG] === "less")) {
                    return __k(Bool["true"]);
                  } else if ((__cps_v_23[LUMO_TAG] === "equal")) {
                    return __k(Bool["true"]);
                  } else {
                    return ((__cps_v_23[LUMO_TAG] === "greater") ? __k(Bool["false"]) : __lumo_match_error(__cps_v_23));
                  }
                });
              } else if ((__cps_v_24[LUMO_TAG] === "false")) {
                return __k(Bool["false"]);
              } else {
                return __lumo_match_error(__cps_v_24);
              }
            };
            if ((__cps_v_25[LUMO_TAG] === "less")) {
              return __k_15(Bool["false"]);
            } else if ((__cps_v_25[LUMO_TAG] === "equal")) {
              return __k_15(Bool["true"]);
            } else {
              return ((__cps_v_25[LUMO_TAG] === "greater") ? __k_15(Bool["true"]) : __lumo_match_error(__cps_v_25));
            }
          });
        } else {
          return __lumo_match_error(__cps_v_27);
        }
      };
      if ((__cps_v_28[LUMO_TAG] === "less")) {
        return __k_17(Bool["false"]);
      } else if ((__cps_v_28[LUMO_TAG] === "equal")) {
        return __k_17(Bool["true"]);
      } else {
        return ((__cps_v_28[LUMO_TAG] === "greater") ? __k_17(Bool["true"]) : __lumo_match_error(__cps_v_28));
      }
    });
  });
}

export function is_ident_start(__caps, c, __k) {
  return is_alpha(__caps, c, __k);
}

export function is_ident_continue(__caps, c, __k) {
  return is_alpha(__caps, c, (__cps_v_30) => {
    if ((__cps_v_30[LUMO_TAG] === "true")) {
      return __k(Bool["true"]);
    } else if ((__cps_v_30[LUMO_TAG] === "false")) {
      return __caps.PartialEq_String.eq(__caps, c, "_", (__cps_v_29) => {
        if ((__cps_v_29[LUMO_TAG] === "true")) {
          return __k(Bool["true"]);
        } else if ((__cps_v_29[LUMO_TAG] === "false")) {
          return __k(Bool["false"]);
        } else {
          return __lumo_match_error(__cps_v_29);
        }
      });
    } else {
      return __lumo_match_error(__cps_v_30);
    }
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

export function state_eof(__caps, st, __k) {
  return __thunk(() => {
    if ((st[LUMO_TAG] === "mk")) {
      const src = st.args[0];
      const pos = st.args[1];
      return String.len(__caps, src, (__cps_v_32) => {
        return __caps.PartialOrd_Number.cmp(__caps, pos, __cps_v_32, (__cps_v_31) => {
          if ((__cps_v_31[LUMO_TAG] === "less")) {
            return __k(Bool["false"]);
          } else if ((__cps_v_31[LUMO_TAG] === "equal")) {
            return __k(Bool["true"]);
          } else {
            return ((__cps_v_31[LUMO_TAG] === "greater") ? __k(Bool["true"]) : __lumo_match_error(__cps_v_31));
          }
        });
      });
    } else {
      return __lumo_match_error(st);
    }
  });
}

export function state_peek(__caps, st, __k) {
  return __thunk(() => {
    if ((st[LUMO_TAG] === "mk")) {
      const src = st.args[0];
      const pos = st.args[1];
      return String.len(__caps, src, (__cps_v_35) => {
        return __caps.PartialOrd_Number.cmp(__caps, pos, __cps_v_35, (__cps_v_34) => {
          const __k_24 = (__cps_v_33) => {
            if ((__cps_v_33[LUMO_TAG] === "true")) {
              return String.char_at(__caps, src, pos, __k);
            } else if ((__cps_v_33[LUMO_TAG] === "false")) {
              return __k("");
            } else {
              return __lumo_match_error(__cps_v_33);
            }
          };
          if ((__cps_v_34[LUMO_TAG] === "less")) {
            return __k_24(Bool["true"]);
          } else if ((__cps_v_34[LUMO_TAG] === "equal")) {
            return __k_24(Bool["false"]);
          } else {
            return ((__cps_v_34[LUMO_TAG] === "greater") ? __k_24(Bool["false"]) : __lumo_match_error(__cps_v_34));
          }
        });
      });
    } else {
      return __lumo_match_error(st);
    }
  });
}

export function state_advance(__caps, st, n, __k) {
  return __thunk(() => {
    if ((st[LUMO_TAG] === "mk")) {
      const src = st.args[0];
      const pos = st.args[1];
      return __caps.Add_Number.add(__caps, pos, n, (__cps_v_36) => {
        return __k(ParseState["mk"](src, __cps_v_36));
      });
    } else {
      return __lumo_match_error(st);
    }
  });
}

export function skip_ws(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_47) => {
    if ((__cps_v_47[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_47[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (c) => {
        return is_whitespace(__caps, c, (__cps_v_46) => {
          if ((__cps_v_46[LUMO_TAG] === "true")) {
            return state_advance(__caps, st, 1, (__cps_v_45) => {
              return skip_ws(__caps, __cps_v_45, __k);
            });
          } else if ((__cps_v_46[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, c, "/", (__cps_v_44) => {
              if ((__cps_v_44[LUMO_TAG] === "true")) {
                return __caps.Add_Number.add(__caps, state_pos(st), 1, (next_pos) => {
                  return String.len(__caps, state_src(st), (__cps_v_43) => {
                    return __caps.PartialOrd_Number.cmp(__caps, next_pos, __cps_v_43, (__cps_v_42) => {
                      const __k_31 = (__cps_v_41) => {
                        if ((__cps_v_41[LUMO_TAG] === "true")) {
                          return String.char_at(__caps, state_src(st), next_pos, (__cps_v_40) => {
                            return __caps.PartialEq_String.eq(__caps, __cps_v_40, "/", (__cps_v_39) => {
                              if ((__cps_v_39[LUMO_TAG] === "true")) {
                                return state_advance(__caps, st, 2, (__cps_v_38) => {
                                  return skip_line(__caps, __cps_v_38, (__cps_v_37) => {
                                    return skip_ws(__caps, __cps_v_37, __k);
                                  });
                                });
                              } else if ((__cps_v_39[LUMO_TAG] === "false")) {
                                return __k(st);
                              } else {
                                return __lumo_match_error(__cps_v_39);
                              }
                            });
                          });
                        } else if ((__cps_v_41[LUMO_TAG] === "false")) {
                          return __k(st);
                        } else {
                          return __lumo_match_error(__cps_v_41);
                        }
                      };
                      if ((__cps_v_42[LUMO_TAG] === "less")) {
                        return __k_31(Bool["true"]);
                      } else if ((__cps_v_42[LUMO_TAG] === "equal")) {
                        return __k_31(Bool["false"]);
                      } else {
                        return ((__cps_v_42[LUMO_TAG] === "greater") ? __k_31(Bool["false"]) : __lumo_match_error(__cps_v_42));
                      }
                    });
                  });
                });
              } else if ((__cps_v_44[LUMO_TAG] === "false")) {
                return __k(st);
              } else {
                return __lumo_match_error(__cps_v_44);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_46);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_47);
    }
  });
}

export function skip_line(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_51) => {
    if ((__cps_v_51[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_51[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__cps_v_50) => {
        return __caps.PartialEq_String.eq(__caps, __cps_v_50, "\n", (__cps_v_49) => {
          if ((__cps_v_49[LUMO_TAG] === "true")) {
            return state_advance(__caps, st, 1, __k);
          } else if ((__cps_v_49[LUMO_TAG] === "false")) {
            return state_advance(__caps, st, 1, (__cps_v_48) => {
              return skip_line(__caps, __cps_v_48, __k);
            });
          } else {
            return __lumo_match_error(__cps_v_49);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_51);
    }
  });
}

export function parse_ident(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_59) => {
      if ((__cps_v_59[LUMO_TAG] === "true")) {
        return __k(ParseResult["err"]("expected identifier, got EOF", state_pos(st2)));
      } else if ((__cps_v_59[LUMO_TAG] === "false")) {
        return state_peek(__caps, st2, (__cps_v_58) => {
          return is_ident_start(__caps, __cps_v_58, (__cps_v_57) => {
            if ((__cps_v_57[LUMO_TAG] === "true")) {
              const start = state_pos(st2);
              return state_advance(__caps, st2, 1, (__cps_v_56) => {
                return scan_ident_rest(__caps, __cps_v_56, (end_st) => {
                  const end_pos = state_pos(end_st);
                  return String.slice(__caps, state_src(st2), start, end_pos, (__cps_v_55) => {
                    return __k(ParseResult["ok"](__cps_v_55, end_st));
                  });
                });
              });
            } else if ((__cps_v_57[LUMO_TAG] === "false")) {
              return state_peek(__caps, st2, (__cps_v_54) => {
                return __caps.Add_String.add(__caps, "expected identifier, got '", __cps_v_54, (__cps_v_53) => {
                  return __caps.Add_String.add(__caps, __cps_v_53, "'", (__cps_v_52) => {
                    return __k(ParseResult["err"](__cps_v_52, state_pos(st2)));
                  });
                });
              });
            } else {
              return __lumo_match_error(__cps_v_57);
            }
          });
        });
      } else {
        return __lumo_match_error(__cps_v_59);
      }
    });
  });
}

export function scan_ident_rest(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_63) => {
    if ((__cps_v_63[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_63[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__cps_v_62) => {
        return is_ident_continue(__caps, __cps_v_62, (__cps_v_61) => {
          if ((__cps_v_61[LUMO_TAG] === "true")) {
            return state_advance(__caps, st, 1, (__cps_v_60) => {
              return scan_ident_rest(__caps, __cps_v_60, __k);
            });
          } else if ((__cps_v_61[LUMO_TAG] === "false")) {
            return __k(st);
          } else {
            return __lumo_match_error(__cps_v_61);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_63);
    }
  });
}

export function expect(__caps, st, expected, __k) {
  return skip_ws(__caps, st, (st2) => {
    return String.len(__caps, expected, (len) => {
      const src = state_src(st2);
      const pos = state_pos(st2);
      return String.len(__caps, src, (__cps_v_75) => {
        return __caps.Sub_Number.sub(__caps, __cps_v_75, pos, (remaining) => {
          return __caps.PartialOrd_Number.cmp(__caps, remaining, len, (__cps_v_74) => {
            const __k_40 = (__cps_v_73) => {
              if ((__cps_v_73[LUMO_TAG] === "true")) {
                return __caps.Add_Number.add(__caps, pos, len, (__cps_v_72) => {
                  return String.slice(__caps, src, pos, __cps_v_72, (slice) => {
                    return __caps.PartialEq_String.eq(__caps, slice, expected, (__cps_v_71) => {
                      if ((__cps_v_71[LUMO_TAG] === "true")) {
                        return state_advance(__caps, st2, len, (__cps_v_70) => {
                          return __k(ParseResult["ok"](expected, __cps_v_70));
                        });
                      } else if ((__cps_v_71[LUMO_TAG] === "false")) {
                        return __caps.Add_String.add(__caps, "expected '", expected, (__cps_v_69) => {
                          return __caps.Add_String.add(__caps, __cps_v_69, "', got '", (__cps_v_68) => {
                            return __caps.Add_String.add(__caps, __cps_v_68, slice, (__cps_v_67) => {
                              return __caps.Add_String.add(__caps, __cps_v_67, "'", (__cps_v_66) => {
                                return __k(ParseResult["err"](__cps_v_66, pos));
                              });
                            });
                          });
                        });
                      } else {
                        return __lumo_match_error(__cps_v_71);
                      }
                    });
                  });
                });
              } else if ((__cps_v_73[LUMO_TAG] === "false")) {
                return __caps.Add_String.add(__caps, "expected '", expected, (__cps_v_65) => {
                  return __caps.Add_String.add(__caps, __cps_v_65, "'", (__cps_v_64) => {
                    return __k(ParseResult["err"](__cps_v_64, pos));
                  });
                });
              } else {
                return __lumo_match_error(__cps_v_73);
              }
            };
            if ((__cps_v_74[LUMO_TAG] === "less")) {
              return __k_40(Bool["false"]);
            } else if ((__cps_v_74[LUMO_TAG] === "equal")) {
              return __k_40(Bool["true"]);
            } else {
              return ((__cps_v_74[LUMO_TAG] === "greater") ? __k_40(Bool["true"]) : __lumo_match_error(__cps_v_74));
            }
          });
        });
      });
    });
  });
}

export function parse_quoted(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__cps_v_79) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_79, "'", (__cps_v_78) => {
        if ((__cps_v_78[LUMO_TAG] === "true")) {
          return __caps.Add_Number.add(__caps, state_pos(st2), 1, (start) => {
            return state_advance(__caps, st2, 1, (__cps_v_77) => {
              return scan_until_quote(__caps, __cps_v_77, (end_st) => {
                const end_pos = state_pos(end_st);
                return String.slice(__caps, state_src(st2), start, end_pos, (content) => {
                  return state_advance(__caps, end_st, 1, (__cps_v_76) => {
                    return __k(ParseResult["ok"](content, __cps_v_76));
                  });
                });
              });
            });
          });
        } else if ((__cps_v_78[LUMO_TAG] === "false")) {
          return __k(ParseResult["err"]("expected quoted literal", state_pos(st2)));
        } else {
          return __lumo_match_error(__cps_v_78);
        }
      });
    });
  });
}

export function scan_until_quote(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_83) => {
    if ((__cps_v_83[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_83[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__cps_v_82) => {
        return __caps.PartialEq_String.eq(__caps, __cps_v_82, "'", (__cps_v_81) => {
          if ((__cps_v_81[LUMO_TAG] === "true")) {
            return __k(st);
          } else if ((__cps_v_81[LUMO_TAG] === "false")) {
            return state_advance(__caps, st, 1, (__cps_v_80) => {
              return scan_until_quote(__caps, __cps_v_80, __k);
            });
          } else {
            return __lumo_match_error(__cps_v_81);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_83);
    }
  });
}

export function peek_char(__caps, st, __k) {
  return skip_ws(__caps, st, (__cps_v_84) => {
    return state_peek(__caps, __cps_v_84, __k);
  });
}

export function peek_is_rule_start(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__cps_v_88) => {
      return is_ident_start(__caps, __cps_v_88, (__cps_v_87) => {
        if ((__cps_v_87[LUMO_TAG] === "true")) {
          return state_advance(__caps, st2, 1, (__cps_v_86) => {
            return scan_ident_rest(__caps, __cps_v_86, (st3) => {
              return skip_ws(__caps, st3, (st4) => {
                return state_peek(__caps, st4, (__cps_v_85) => {
                  return __caps.PartialEq_String.eq(__caps, __cps_v_85, "=", __k);
                });
              });
            });
          });
        } else if ((__cps_v_87[LUMO_TAG] === "false")) {
          return __k(Bool["false"]);
        } else {
          return __lumo_match_error(__cps_v_87);
        }
      });
    });
  });
}

export function classify_literal(__caps, text, __k) {
  return has_alpha(__caps, text, 0, (__cps_v_89) => {
    if ((__cps_v_89[LUMO_TAG] === "true")) {
      return __k(TokenRef["keyword"](text));
    } else if ((__cps_v_89[LUMO_TAG] === "false")) {
      return __k(TokenRef["symbol"](text));
    } else {
      return __lumo_match_error(__cps_v_89);
    }
  });
}

export function has_alpha(__caps, s, i, __k) {
  return String.len(__caps, s, (__cps_v_95) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_95, (__cps_v_94) => {
      const __k_48 = (__cps_v_93) => {
        if ((__cps_v_93[LUMO_TAG] === "true")) {
          return __k(Bool["false"]);
        } else if ((__cps_v_93[LUMO_TAG] === "false")) {
          return String.char_at(__caps, s, i, (__cps_v_92) => {
            return is_alpha(__caps, __cps_v_92, (__cps_v_91) => {
              if ((__cps_v_91[LUMO_TAG] === "true")) {
                return __k(Bool["true"]);
              } else if ((__cps_v_91[LUMO_TAG] === "false")) {
                return __caps.Add_Number.add(__caps, i, 1, (__cps_v_90) => {
                  return has_alpha(__caps, s, __cps_v_90, __k);
                });
              } else {
                return __lumo_match_error(__cps_v_91);
              }
            });
          });
        } else {
          return __lumo_match_error(__cps_v_93);
        }
      };
      if ((__cps_v_94[LUMO_TAG] === "less")) {
        return __k_48(Bool["false"]);
      } else if ((__cps_v_94[LUMO_TAG] === "equal")) {
        return __k_48(Bool["true"]);
      } else {
        return ((__cps_v_94[LUMO_TAG] === "greater") ? __k_48(Bool["true"]) : __lumo_match_error(__cps_v_94));
      }
    });
  });
}

export function parse_grammar(__caps, src, __k) {
  return __thunk(() => {
    const st = ParseState["mk"](src, 0);
    return parse_grammar_items(__caps, st, List["nil"], List["nil"], __k);
  });
}

export function parse_grammar_items(__caps, st, tokens, rules, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_100) => {
      if ((__cps_v_100[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_rule(rules)), st2));
      } else if ((__cps_v_100[LUMO_TAG] === "false")) {
        return state_peek(__caps, st2, (__cps_v_99) => {
          return __caps.PartialEq_String.eq(__caps, __cps_v_99, "@", (__cps_v_98) => {
            if ((__cps_v_98[LUMO_TAG] === "true")) {
              return parse_token_def(__caps, st2, (__cps_v_97) => {
                if ((__cps_v_97[LUMO_TAG] === "ok")) {
                  const new_tokens = __cps_v_97.args[0];
                  const st3 = __cps_v_97.args[1];
                  return parse_grammar_items(__caps, st3, list_concat_string(new_tokens, tokens), rules, __k);
                } else if ((__cps_v_97[LUMO_TAG] === "err")) {
                  const msg = __cps_v_97.args[0];
                  const pos = __cps_v_97.args[1];
                  return __k(ParseResult["err"](msg, pos));
                } else {
                  return __lumo_match_error(__cps_v_97);
                }
              });
            } else if ((__cps_v_98[LUMO_TAG] === "false")) {
              return parse_rule(__caps, st2, (__cps_v_96) => {
                if ((__cps_v_96[LUMO_TAG] === "ok")) {
                  const rule = __cps_v_96.args[0];
                  const st3 = __cps_v_96.args[1];
                  return parse_grammar_items(__caps, st3, tokens, List["cons"](rule, rules), __k);
                } else if ((__cps_v_96[LUMO_TAG] === "err")) {
                  const msg = __cps_v_96.args[0];
                  const pos = __cps_v_96.args[1];
                  return __k(ParseResult["err"](msg, pos));
                } else {
                  return __lumo_match_error(__cps_v_96);
                }
              });
            } else {
              return __lumo_match_error(__cps_v_98);
            }
          });
        });
      } else {
        return __lumo_match_error(__cps_v_100);
      }
    });
  });
}

export function parse_token_def(__caps, st, __k) {
  return expect(__caps, st, "@token", (__cps_v_101) => {
    if ((__cps_v_101[LUMO_TAG] === "err")) {
      const msg = __cps_v_101.args[0];
      const pos = __cps_v_101.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_101[LUMO_TAG] === "ok")) {
      const st2 = __cps_v_101.args[1];
      return parse_token_names(__caps, st2, List["nil"], __k);
    } else {
      return __lumo_match_error(__cps_v_101);
    }
  });
}

export function parse_token_names(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_108) => {
      if ((__cps_v_108[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](list_reverse_string(acc), st2));
      } else if ((__cps_v_108[LUMO_TAG] === "false")) {
        return peek_is_rule_start(__caps, st2, (__cps_v_107) => {
          if ((__cps_v_107[LUMO_TAG] === "true")) {
            return __k(ParseResult["ok"](list_reverse_string(acc), st2));
          } else if ((__cps_v_107[LUMO_TAG] === "false")) {
            return state_peek(__caps, st2, (__cps_v_106) => {
              return __caps.PartialEq_String.eq(__caps, __cps_v_106, "@", (__cps_v_105) => {
                if ((__cps_v_105[LUMO_TAG] === "true")) {
                  return __k(ParseResult["ok"](list_reverse_string(acc), st2));
                } else if ((__cps_v_105[LUMO_TAG] === "false")) {
                  return state_peek(__caps, st2, (__cps_v_104) => {
                    return is_ident_start(__caps, __cps_v_104, (__cps_v_103) => {
                      if ((__cps_v_103[LUMO_TAG] === "true")) {
                        return parse_ident(__caps, st2, (__cps_v_102) => {
                          if ((__cps_v_102[LUMO_TAG] === "ok")) {
                            const name = __cps_v_102.args[0];
                            const st3 = __cps_v_102.args[1];
                            return parse_token_names(__caps, st3, List["cons"](name, acc), __k);
                          } else if ((__cps_v_102[LUMO_TAG] === "err")) {
                            const msg = __cps_v_102.args[0];
                            const pos = __cps_v_102.args[1];
                            return __k(ParseResult["err"](msg, pos));
                          } else {
                            return __lumo_match_error(__cps_v_102);
                          }
                        });
                      } else if ((__cps_v_103[LUMO_TAG] === "false")) {
                        return __k(ParseResult["ok"](list_reverse_string(acc), st2));
                      } else {
                        return __lumo_match_error(__cps_v_103);
                      }
                    });
                  });
                } else {
                  return __lumo_match_error(__cps_v_105);
                }
              });
            });
          } else {
            return __lumo_match_error(__cps_v_107);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_108);
      }
    });
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident(__caps, st, (__cps_v_111) => {
    if ((__cps_v_111[LUMO_TAG] === "err")) {
      const msg = __cps_v_111.args[0];
      const pos = __cps_v_111.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_111[LUMO_TAG] === "ok")) {
      const name = __cps_v_111.args[0];
      const st2 = __cps_v_111.args[1];
      return expect(__caps, st2, "=", (__cps_v_110) => {
        if ((__cps_v_110[LUMO_TAG] === "err")) {
          const msg = __cps_v_110.args[0];
          const pos = __cps_v_110.args[1];
          return __k(ParseResult["err"](msg, pos));
        } else if ((__cps_v_110[LUMO_TAG] === "ok")) {
          const st3 = __cps_v_110.args[1];
          return parse_rule_body(__caps, st3, name, (__cps_v_109) => {
            if ((__cps_v_109[LUMO_TAG] === "err")) {
              const msg = __cps_v_109.args[0];
              const pos = __cps_v_109.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else if ((__cps_v_109[LUMO_TAG] === "ok")) {
              const body = __cps_v_109.args[0];
              const st4 = __cps_v_109.args[1];
              return __k(ParseResult["ok"](Rule["mk"](name, body), st4));
            } else {
              return __lumo_match_error(__cps_v_109);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_110);
        }
      });
    } else {
      return __lumo_match_error(__cps_v_111);
    }
  });
}

export function parse_rule_body(__caps, st, rule_name, __k) {
  return skip_ws(__caps, st, (st2) => {
    return peek_char(__caps, st2, (__cps_v_113) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_113, "|", (__cps_v_112) => {
        if ((__cps_v_112[LUMO_TAG] === "true")) {
          return parse_alternatives(__caps, st2, __k);
        } else if ((__cps_v_112[LUMO_TAG] === "false")) {
          return parse_sequence(__caps, st2, __k);
        } else {
          return __lumo_match_error(__cps_v_112);
        }
      });
    });
  });
}

export function parse_alternatives(__caps, st, __k) {
  return parse_alt_items(__caps, st, List["nil"], __k);
}

export function parse_alt_items(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return peek_char(__caps, st2, (__cps_v_120) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_120, "|", (__cps_v_119) => {
        if ((__cps_v_119[LUMO_TAG] === "true")) {
          return skip_ws(__caps, st2, (__cps_v_118) => {
            return state_advance(__caps, __cps_v_118, 1, (st3) => {
              return skip_ws(__caps, st3, (st4) => {
                return state_peek(__caps, st4, (__cps_v_117) => {
                  return __caps.PartialEq_String.eq(__caps, __cps_v_117, "'", (__cps_v_116) => {
                    if ((__cps_v_116[LUMO_TAG] === "true")) {
                      return parse_quoted(__caps, st4, (__cps_v_115) => {
                        if ((__cps_v_115[LUMO_TAG] === "ok")) {
                          const lit = __cps_v_115.args[0];
                          const st5 = __cps_v_115.args[1];
                          return parse_alt_items(__caps, st5, List["cons"](Alternative["mk"](lit), acc), __k);
                        } else if ((__cps_v_115[LUMO_TAG] === "err")) {
                          const msg = __cps_v_115.args[0];
                          const pos = __cps_v_115.args[1];
                          return __k(ParseResult["err"](msg, pos));
                        } else {
                          return __lumo_match_error(__cps_v_115);
                        }
                      });
                    } else if ((__cps_v_116[LUMO_TAG] === "false")) {
                      return parse_ident(__caps, st3, (__cps_v_114) => {
                        if ((__cps_v_114[LUMO_TAG] === "ok")) {
                          const name = __cps_v_114.args[0];
                          const st5 = __cps_v_114.args[1];
                          return parse_alt_items(__caps, st5, List["cons"](Alternative["mk"](name), acc), __k);
                        } else if ((__cps_v_114[LUMO_TAG] === "err")) {
                          const msg = __cps_v_114.args[0];
                          const pos = __cps_v_114.args[1];
                          return __k(ParseResult["err"](msg, pos));
                        } else {
                          return __lumo_match_error(__cps_v_114);
                        }
                      });
                    } else {
                      return __lumo_match_error(__cps_v_116);
                    }
                  });
                });
              });
            });
          });
        } else if ((__cps_v_119[LUMO_TAG] === "false")) {
          return __k(ParseResult["ok"](RuleBody["alternatives"](list_reverse_alt(acc)), st2));
        } else {
          return __lumo_match_error(__cps_v_119);
        }
      });
    });
  });
}

export function parse_sequence(__caps, st, __k) {
  return parse_seq_elements(__caps, st, List["nil"], __k);
}

export function parse_seq_elements(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_123) => {
      if ((__cps_v_123[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
      } else if ((__cps_v_123[LUMO_TAG] === "false")) {
        return is_seq_terminator(__caps, st2, (__cps_v_122) => {
          if ((__cps_v_122[LUMO_TAG] === "true")) {
            return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
          } else if ((__cps_v_122[LUMO_TAG] === "false")) {
            return parse_element(__caps, st2, (__cps_v_121) => {
              if ((__cps_v_121[LUMO_TAG] === "ok")) {
                const elem = __cps_v_121.args[0];
                const st3 = __cps_v_121.args[1];
                return parse_seq_elements(__caps, st3, List["cons"](elem, acc), __k);
              } else if ((__cps_v_121[LUMO_TAG] === "err")) {
                const msg = __cps_v_121.args[0];
                const pos = __cps_v_121.args[1];
                return __k(ParseResult["err"](msg, pos));
              } else {
                return __lumo_match_error(__cps_v_121);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_122);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_123);
      }
    });
  });
}

export function is_seq_terminator(__caps, st, __k) {
  return peek_char(__caps, st, (c) => {
    return __caps.PartialEq_String.eq(__caps, c, ")", (__cps_v_126) => {
      if ((__cps_v_126[LUMO_TAG] === "true")) {
        return __k(Bool["true"]);
      } else if ((__cps_v_126[LUMO_TAG] === "false")) {
        return peek_is_rule_start(__caps, st, (__cps_v_125) => {
          if ((__cps_v_125[LUMO_TAG] === "true")) {
            return __k(Bool["true"]);
          } else if ((__cps_v_125[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, c, "@", (__cps_v_124) => {
              if ((__cps_v_124[LUMO_TAG] === "true")) {
                return __k(Bool["true"]);
              } else if ((__cps_v_124[LUMO_TAG] === "false")) {
                return __k(Bool["false"]);
              } else {
                return __lumo_match_error(__cps_v_124);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_125);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_126);
      }
    });
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom(__caps, st, (__cps_v_127) => {
    if ((__cps_v_127[LUMO_TAG] === "err")) {
      const msg = __cps_v_127.args[0];
      const pos = __cps_v_127.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_127[LUMO_TAG] === "ok")) {
      const elem = __cps_v_127.args[0];
      const st2 = __cps_v_127.args[1];
      return apply_postfix_elem(__caps, elem, st2, __k);
    } else {
      return __lumo_match_error(__cps_v_127);
    }
  });
}

export function apply_postfix_elem(__caps, elem, st, __k) {
  return state_eof(__caps, st, (__cps_v_134) => {
    if ((__cps_v_134[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](elem, st));
    } else if ((__cps_v_134[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__cps_v_133) => {
        return __caps.PartialEq_String.eq(__caps, __cps_v_133, "?", (__cps_v_132) => {
          if ((__cps_v_132[LUMO_TAG] === "true")) {
            return state_advance(__caps, st, 1, (__cps_v_131) => {
              return apply_postfix_elem(__caps, Element["optional"](elem), __cps_v_131, __k);
            });
          } else if ((__cps_v_132[LUMO_TAG] === "false")) {
            return state_peek(__caps, st, (__cps_v_130) => {
              return __caps.PartialEq_String.eq(__caps, __cps_v_130, "*", (__cps_v_129) => {
                if ((__cps_v_129[LUMO_TAG] === "true")) {
                  return state_advance(__caps, st, 1, (__cps_v_128) => {
                    return apply_postfix_elem(__caps, Element["repeated"](elem), __cps_v_128, __k);
                  });
                } else if ((__cps_v_129[LUMO_TAG] === "false")) {
                  return __k(ParseResult["ok"](elem, st));
                } else {
                  return __lumo_match_error(__cps_v_129);
                }
              });
            });
          } else {
            return __lumo_match_error(__cps_v_132);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_134);
    }
  });
}

export function parse_atom(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__cps_v_147) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_147, "'", (__cps_v_146) => {
        if ((__cps_v_146[LUMO_TAG] === "true")) {
          return parse_quoted(__caps, st2, (__cps_v_145) => {
            if ((__cps_v_145[LUMO_TAG] === "ok")) {
              const lit = __cps_v_145.args[0];
              const st3 = __cps_v_145.args[1];
              return classify_literal(__caps, lit, (__cps_v_144) => {
                const __cps_v_143 = Element["token"](__cps_v_144);
                return __k(ParseResult["ok"](__cps_v_143, st3));
              });
            } else if ((__cps_v_145[LUMO_TAG] === "err")) {
              const msg = __cps_v_145.args[0];
              const pos = __cps_v_145.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_145);
            }
          });
        } else if ((__cps_v_146[LUMO_TAG] === "false")) {
          return state_peek(__caps, st2, (__cps_v_142) => {
            return __caps.PartialEq_String.eq(__caps, __cps_v_142, "(", (__cps_v_141) => {
              if ((__cps_v_141[LUMO_TAG] === "true")) {
                return state_advance(__caps, st2, 1, (st3) => {
                  return parse_group_elements(__caps, st3, List["nil"], (__cps_v_140) => {
                    if ((__cps_v_140[LUMO_TAG] === "ok")) {
                      const elems = __cps_v_140.args[0];
                      const st4 = __cps_v_140.args[1];
                      return expect(__caps, st4, ")", (__cps_v_139) => {
                        if ((__cps_v_139[LUMO_TAG] === "ok")) {
                          const st5 = __cps_v_139.args[1];
                          return __k(ParseResult["ok"](Element["group"](elems), st5));
                        } else if ((__cps_v_139[LUMO_TAG] === "err")) {
                          const msg = __cps_v_139.args[0];
                          const pos = __cps_v_139.args[1];
                          return __k(ParseResult["err"](msg, pos));
                        } else {
                          return __lumo_match_error(__cps_v_139);
                        }
                      });
                    } else if ((__cps_v_140[LUMO_TAG] === "err")) {
                      const msg = __cps_v_140.args[0];
                      const pos = __cps_v_140.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_140);
                    }
                  });
                });
              } else if ((__cps_v_141[LUMO_TAG] === "false")) {
                return parse_ident(__caps, st2, (__cps_v_138) => {
                  if ((__cps_v_138[LUMO_TAG] === "err")) {
                    const msg = __cps_v_138.args[0];
                    const pos = __cps_v_138.args[1];
                    return __k(ParseResult["err"](msg, pos));
                  } else if ((__cps_v_138[LUMO_TAG] === "ok")) {
                    const name = __cps_v_138.args[0];
                    const st3 = __cps_v_138.args[1];
                    return state_peek(__caps, st3, (__cps_v_137) => {
                      return __caps.PartialEq_String.eq(__caps, __cps_v_137, ":", (__cps_v_136) => {
                        if ((__cps_v_136[LUMO_TAG] === "true")) {
                          return state_advance(__caps, st3, 1, (st4) => {
                            return parse_element(__caps, st4, (__cps_v_135) => {
                              if ((__cps_v_135[LUMO_TAG] === "ok")) {
                                const inner = __cps_v_135.args[0];
                                const st5 = __cps_v_135.args[1];
                                return __k(ParseResult["ok"](Element["labeled"](name, inner), st5));
                              } else if ((__cps_v_135[LUMO_TAG] === "err")) {
                                const msg = __cps_v_135.args[0];
                                const pos = __cps_v_135.args[1];
                                return __k(ParseResult["err"](msg, pos));
                              } else {
                                return __lumo_match_error(__cps_v_135);
                              }
                            });
                          });
                        } else if ((__cps_v_136[LUMO_TAG] === "false")) {
                          return __k(ParseResult["ok"](Element["node"](NodeRef["mk"](name)), st3));
                        } else {
                          return __lumo_match_error(__cps_v_136);
                        }
                      });
                    });
                  } else {
                    return __lumo_match_error(__cps_v_138);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_141);
              }
            });
          });
        } else {
          return __lumo_match_error(__cps_v_146);
        }
      });
    });
  });
}

export function parse_group_elements(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__cps_v_150) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_150, ")", (__cps_v_149) => {
        if ((__cps_v_149[LUMO_TAG] === "true")) {
          return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
        } else if ((__cps_v_149[LUMO_TAG] === "false")) {
          return parse_element(__caps, st2, (__cps_v_148) => {
            if ((__cps_v_148[LUMO_TAG] === "ok")) {
              const elem = __cps_v_148.args[0];
              const st3 = __cps_v_148.args[1];
              return parse_group_elements(__caps, st3, List["cons"](elem, acc), __k);
            } else if ((__cps_v_148[LUMO_TAG] === "err")) {
              const msg = __cps_v_148.args[0];
              const pos = __cps_v_148.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_148);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_149);
        }
      });
    });
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    if ((g[LUMO_TAG] === "mk")) {
      const token_defs = g.args[0];
      const rules = g.args[1];
      return resolve_rules(__caps, token_defs, rules, (__cps_v_151) => {
        return __k(Grammar["mk"](token_defs, __cps_v_151));
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
          return resolve_rules(__caps, token_defs, rest, (__cps_v_152) => {
            return __k(List["cons"](Rule["mk"](name, resolved_body), __cps_v_152));
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
      return resolve_elements(__caps, token_defs, elems, (__cps_v_153) => {
        return __k(RuleBody["sequence"](__cps_v_153));
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
      return resolve_element(__caps, token_defs, elem, (__cps_v_154) => {
        return resolve_elements(__caps, token_defs, rest, (__cps_v_155) => {
          return __k(List["cons"](__cps_v_154, __cps_v_155));
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
        return list_contains_string(__caps, token_defs, name, (__cps_v_160) => {
          if ((__cps_v_160[LUMO_TAG] === "true")) {
            return __k(Element["token"](TokenRef["named"](name)));
          } else if ((__cps_v_160[LUMO_TAG] === "false")) {
            return __k(elem);
          } else {
            return __lumo_match_error(__cps_v_160);
          }
        });
      } else {
        return __lumo_match_error(ref);
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        const inner = elem.args[1];
        return resolve_element(__caps, token_defs, inner, (__cps_v_159) => {
          return __k(Element["labeled"](label, __cps_v_159));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_158) => {
          return __k(Element["optional"](__cps_v_158));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_157) => {
          return __k(Element["repeated"](__cps_v_157));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_156) => {
          return __k(Element["group"](__cps_v_156));
        });
      })(elem.args[0]) : __lumo_match_error(elem)))));
    }
  });
}

export function list_contains_string(__caps, xs, target, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(Bool["false"]);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const x = xs.args[0];
      const rest = xs.args[1];
      return __caps.PartialEq_String.eq(__caps, x, target, (__cps_v_161) => {
        if ((__cps_v_161[LUMO_TAG] === "true")) {
          return __k(Bool["true"]);
        } else if ((__cps_v_161[LUMO_TAG] === "false")) {
          return list_contains_string(__caps, rest, target, __k);
        } else {
          return __lumo_match_error(__cps_v_161);
        }
      });
    } else {
      return __lumo_match_error(xs);
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


export const Bool = { "true": { [LUMO_TAG]: "true" }, "false": { [LUMO_TAG]: "false" } };


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

export function __println(msg) {
  return globalThis.console.log(msg);
}

export const IO = { println: (__caps, msg, __k) => {
  return __thunk(() => {
    return __k(__println(msg));
  });
} };

export function readFileSync(path, encoding) {
  return __lumo_readFileSync(path, encoding);
}

export function writeFileSync(path, content, encoding) {
  return __lumo_writeFileSync(path, content, encoding);
}

export const FS = { read_file: (__caps, path, __k) => {
  return __thunk(() => {
    return __k(readFileSync(path, "utf8"));
  });
}, write_file: (__caps, path, content, __k) => {
  return __thunk(() => {
    return __k(writeFileSync(path, content, "utf8"));
  });
} };


export const Ordering = { "less": { [LUMO_TAG]: "less" }, "equal": { [LUMO_TAG]: "equal" }, "greater": { [LUMO_TAG]: "greater" } };

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

export function __argv_at_offset(__caps, idx, __k) {
  return __thunk(() => {
    return __caps.Add_Number.add(__caps, idx, 1, (__cps_v_162) => {
      return __k(__argv_at_raw(__cps_v_162));
    });
  });
}

export function __args_count_offset(__caps, __k) {
  return __thunk(() => {
    return __caps.Sub_Number.sub(__caps, __argv_length_raw(), 1, __k);
  });
}

export const Process = { arg_at: (__caps, idx, __k) => {
  return __argv_at_offset(__caps, idx, __k);
}, args_count: (__caps, __k) => {
  return __args_count_offset(__caps, __k);
}, exit_process: (__caps, code, __k) => {
  return __thunk(() => {
    return __k(__exit_process(code));
  });
}, panic_with: (__caps, msg, __k) => {
  return __thunk(() => {
    const _err = __console_error(msg);
    return __k(__exit_process(1));
  });
} };

export const __impl_Bool_Not = { not: (__caps, self, __k) => {
  return __thunk(() => {
    if ((self[LUMO_TAG] === "true")) {
      return __k(Bool["false"]);
    } else if ((self[LUMO_TAG] === "false")) {
      return __k(Bool["true"]);
    } else {
      return __lumo_match_error(self);
    }
  });
} };

export const __impl_Number_PartialEq = { eq: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.eq(__caps, self, other, __k);
  });
} };

export const __impl_Number_PartialOrd = { cmp: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.cmp(__caps, self, other, __k);
  });
} };

export const __impl_Number_Add = { add: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.add(__caps, self, other, __k);
  });
} };

export const __impl_Number_Sub = { sub: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.sub(__caps, self, other, __k);
  });
} };

export const __impl_Number_Mul = { mul: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.mul(__caps, self, other, __k);
  });
} };

export const __impl_Number_Div = { div: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.div(__caps, self, other, __k);
  });
} };

export const __impl_Number_Mod = { mod_: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.mod_(__caps, self, other, __k);
  });
} };

export const __impl_Number_Neg = { neg: (__caps, self, __k) => {
  return __thunk(() => {
    return __caps.NumOps_NumOps.neg(__caps, self, __k);
  });
} };

export function __num_add(a, b) {
  return (a + b);
}

export function __num_sub(a, b) {
  return (a - b);
}

export function __num_mul(a, b) {
  return (a * b);
}

export function __num_div(a, b) {
  return (a / b);
}

export function __num_mod(a, b) {
  return globalThis["_%_"];
}

export function __num_neg(a) {
  return (-a);
}

export function __num_floor(a) {
  return Math.floor(a);
}

export function __num_eq(a, b) {
  if ((a === b)) {
    return Bool["true"];
  } else {
    return Bool["false"];
  }
}

export function __num_lt(a, b) {
  if ((a < b)) {
    return Bool["true"];
  } else {
    return Bool["false"];
  }
}

export const NumOps = { add: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_add(a, b));
  });
}, sub: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_sub(a, b));
  });
}, mul: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_mul(a, b));
  });
}, div: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_div(a, b));
  });
}, mod_: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_mod(a, b));
  });
}, neg: (__caps, a, __k) => {
  return __thunk(() => {
    return __k(__num_neg(a));
  });
}, floor: (__caps, a, __k) => {
  return __thunk(() => {
    return __k(__num_floor(a));
  });
}, eq: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__num_eq(a, b));
  });
}, cmp: (__caps, a, b, __k) => {
  return __thunk(() => {
    const __match_107 = __num_lt(a, b);
    if ((__match_107[LUMO_TAG] === "true")) {
      return __k(Ordering["less"]);
    } else if ((__match_107[LUMO_TAG] === "false")) {
      const __match_108 = __num_eq(a, b);
      if ((__match_108[LUMO_TAG] === "true")) {
        return __k(Ordering["equal"]);
      } else if ((__match_108[LUMO_TAG] === "false")) {
        return __k(Ordering["greater"]);
      } else {
        return __lumo_match_error(__match_108);
      }
    } else {
      return __lumo_match_error(__match_107);
    }
  });
} };

export const __impl_String_Add = { add: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.concat(__caps, self, other, __k);
  });
} };

export const __impl_String_PartialEq = { eq: (__caps, self, other, __k) => {
  return __thunk(() => {
    return __caps.StrOps_StrOps.eq(__caps, self, other, __k);
  });
} };

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

export function __str_len(s) {
  return s.length;
}

export function __str_char_at(s, idx) {
  return s.charAt(idx);
}

export function __str_slice(s, start, end) {
  return s.slice(start, end);
}

export function __str_starts_with(s, prefix) {
  if (s.startsWith(prefix)) {
    return Bool["true"];
  } else {
    return Bool["false"];
  }
}

export function __str_contains(s, sub) {
  if (s.includes(sub)) {
    return Bool["true"];
  } else {
    return Bool["false"];
  }
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

export function __str_concat(a, b) {
  return (a + b);
}

export function __str_eq(a, b) {
  if ((a === b)) {
    return Bool["true"];
  } else {
    return Bool["false"];
  }
}

export const StrOps = { len: (__caps, s, __k) => {
  return __thunk(() => {
    return __k(__str_len(s));
  });
}, char_at: (__caps, s, idx, __k) => {
  return __thunk(() => {
    return __k(__str_char_at(s, idx));
  });
}, slice: (__caps, s, start, end, __k) => {
  return __thunk(() => {
    return __k(__str_slice(s, start, end));
  });
}, concat: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__str_concat(a, b));
  });
}, eq: (__caps, a, b, __k) => {
  return __thunk(() => {
    return __k(__str_eq(a, b));
  });
}, starts_with: (__caps, s, prefix, __k) => {
  return __thunk(() => {
    return __k(__str_starts_with(s, prefix));
  });
}, contains: (__caps, s, sub, __k) => {
  return __thunk(() => {
    return __k(__str_contains(s, sub));
  });
}, index_of: (__caps, s, sub, __k) => {
  return __thunk(() => {
    return __k(__str_index_of(s, sub));
  });
}, trim: (__caps, s, __k) => {
  return __thunk(() => {
    return __k(__str_trim(s));
  });
}, char_code_at: (__caps, s, idx, __k) => {
  return __thunk(() => {
    return __k(__char_code_at(s, idx));
  });
}, from_char_code: (__caps, code, __k) => {
  return __thunk(() => {
    return __k(fromCharCode(code));
  });
}, replace_all: (__caps, s, from, to, __k) => {
  return __thunk(() => {
    return __k(__str_replace_all(s, from, to));
  });
}, num_to_string: (__caps, n, __k) => {
  return __thunk(() => {
    return __k(__num_to_string(n));
  });
} };

export function to_screaming_snake(__caps, name, __k) {
  return to_screaming_snake_loop(__caps, name, 0, "", __k);
}

export function to_screaming_snake_loop(__caps, name, i, acc, __k) {
  return String.len(__caps, name, (__cps_v_195) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_195, (__cps_v_194) => {
      const __k_115 = (__cps_v_193) => {
        if ((__cps_v_193[LUMO_TAG] === "true")) {
          return __k(acc);
        } else if ((__cps_v_193[LUMO_TAG] === "false")) {
          return String.char_at(__caps, name, i, (c) => {
            return String.char_code_at(__caps, c, 0, (code) => {
              return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_192) => {
                const __k_114 = (__cps_v_191) => {
                  const __k_112 = (is_upper) => {
                    if ((is_upper[LUMO_TAG] === "true")) {
                      return __caps.PartialOrd_Number.cmp(__caps, 0, i, (__cps_v_189) => {
                        const __k_111 = (__cps_v_188) => {
                          if ((__cps_v_188[LUMO_TAG] === "true")) {
                            return __caps.Sub_Number.sub(__caps, i, 1, (__cps_v_187) => {
                              return String.char_at(__caps, name, __cps_v_187, (__cps_v_186) => {
                                return String.char_code_at(__caps, __cps_v_186, 0, (prev_code) => {
                                  return __caps.PartialOrd_Number.cmp(__caps, prev_code, 97, (__cps_v_185) => {
                                    const __k_110 = (__cps_v_184) => {
                                      const __k_108 = (prev_lower) => {
                                        return __caps.PartialOrd_Number.cmp(__caps, prev_code, 48, (__cps_v_182) => {
                                          const __k_107 = (__cps_v_181) => {
                                            const __k_105 = (prev_digit) => {
                                              if ((prev_lower[LUMO_TAG] === "true")) {
                                                return __caps.Add_Number.add(__caps, i, 1, (__cps_v_176) => {
                                                  return __caps.Add_String.add(__caps, acc, "_", (__cps_v_178) => {
                                                    return to_upper_char(__caps, c, (__cps_v_179) => {
                                                      return __caps.Add_String.add(__caps, __cps_v_178, __cps_v_179, (__cps_v_177) => {
                                                        return to_screaming_snake_loop(__caps, name, __cps_v_176, __cps_v_177, __k);
                                                      });
                                                    });
                                                  });
                                                });
                                              } else if ((prev_lower[LUMO_TAG] === "false")) {
                                                if ((prev_digit[LUMO_TAG] === "true")) {
                                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_172) => {
                                                    return __caps.Add_String.add(__caps, acc, "_", (__cps_v_174) => {
                                                      return to_upper_char(__caps, c, (__cps_v_175) => {
                                                        return __caps.Add_String.add(__caps, __cps_v_174, __cps_v_175, (__cps_v_173) => {
                                                          return to_screaming_snake_loop(__caps, name, __cps_v_172, __cps_v_173, __k);
                                                        });
                                                      });
                                                    });
                                                  });
                                                } else if ((prev_digit[LUMO_TAG] === "false")) {
                                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_169) => {
                                                    return to_upper_char(__caps, c, (__cps_v_171) => {
                                                      return __caps.Add_String.add(__caps, acc, __cps_v_171, (__cps_v_170) => {
                                                        return to_screaming_snake_loop(__caps, name, __cps_v_169, __cps_v_170, __k);
                                                      });
                                                    });
                                                  });
                                                } else {
                                                  return __lumo_match_error(prev_digit);
                                                }
                                              } else {
                                                return __lumo_match_error(prev_lower);
                                              }
                                            };
                                            if ((__cps_v_181[LUMO_TAG] === "true")) {
                                              return __caps.PartialOrd_Number.cmp(__caps, prev_code, 57, (__cps_v_180) => {
                                                if ((__cps_v_180[LUMO_TAG] === "less")) {
                                                  return __k_105(Bool["true"]);
                                                } else if ((__cps_v_180[LUMO_TAG] === "equal")) {
                                                  return __k_105(Bool["true"]);
                                                } else {
                                                  return ((__cps_v_180[LUMO_TAG] === "greater") ? __k_105(Bool["false"]) : __lumo_match_error(__cps_v_180));
                                                }
                                              });
                                            } else if ((__cps_v_181[LUMO_TAG] === "false")) {
                                              return __k_105(Bool["false"]);
                                            } else {
                                              return __lumo_match_error(__cps_v_181);
                                            }
                                          };
                                          if ((__cps_v_182[LUMO_TAG] === "less")) {
                                            return __k_107(Bool["false"]);
                                          } else if ((__cps_v_182[LUMO_TAG] === "equal")) {
                                            return __k_107(Bool["true"]);
                                          } else {
                                            return ((__cps_v_182[LUMO_TAG] === "greater") ? __k_107(Bool["true"]) : __lumo_match_error(__cps_v_182));
                                          }
                                        });
                                      };
                                      if ((__cps_v_184[LUMO_TAG] === "true")) {
                                        return __caps.PartialOrd_Number.cmp(__caps, prev_code, 122, (__cps_v_183) => {
                                          if ((__cps_v_183[LUMO_TAG] === "less")) {
                                            return __k_108(Bool["true"]);
                                          } else if ((__cps_v_183[LUMO_TAG] === "equal")) {
                                            return __k_108(Bool["true"]);
                                          } else {
                                            return ((__cps_v_183[LUMO_TAG] === "greater") ? __k_108(Bool["false"]) : __lumo_match_error(__cps_v_183));
                                          }
                                        });
                                      } else if ((__cps_v_184[LUMO_TAG] === "false")) {
                                        return __k_108(Bool["false"]);
                                      } else {
                                        return __lumo_match_error(__cps_v_184);
                                      }
                                    };
                                    if ((__cps_v_185[LUMO_TAG] === "less")) {
                                      return __k_110(Bool["false"]);
                                    } else if ((__cps_v_185[LUMO_TAG] === "equal")) {
                                      return __k_110(Bool["true"]);
                                    } else {
                                      return ((__cps_v_185[LUMO_TAG] === "greater") ? __k_110(Bool["true"]) : __lumo_match_error(__cps_v_185));
                                    }
                                  });
                                });
                              });
                            });
                          } else if ((__cps_v_188[LUMO_TAG] === "false")) {
                            return __caps.Add_Number.add(__caps, i, 1, (__cps_v_166) => {
                              return to_upper_char(__caps, c, (__cps_v_168) => {
                                return __caps.Add_String.add(__caps, acc, __cps_v_168, (__cps_v_167) => {
                                  return to_screaming_snake_loop(__caps, name, __cps_v_166, __cps_v_167, __k);
                                });
                              });
                            });
                          } else {
                            return __lumo_match_error(__cps_v_188);
                          }
                        };
                        if ((__cps_v_189[LUMO_TAG] === "less")) {
                          return __k_111(Bool["true"]);
                        } else if ((__cps_v_189[LUMO_TAG] === "equal")) {
                          return __k_111(Bool["false"]);
                        } else {
                          return ((__cps_v_189[LUMO_TAG] === "greater") ? __k_111(Bool["false"]) : __lumo_match_error(__cps_v_189));
                        }
                      });
                    } else if ((is_upper[LUMO_TAG] === "false")) {
                      return __caps.Add_Number.add(__caps, i, 1, (__cps_v_163) => {
                        return to_upper_char(__caps, c, (__cps_v_165) => {
                          return __caps.Add_String.add(__caps, acc, __cps_v_165, (__cps_v_164) => {
                            return to_screaming_snake_loop(__caps, name, __cps_v_163, __cps_v_164, __k);
                          });
                        });
                      });
                    } else {
                      return __lumo_match_error(is_upper);
                    }
                  };
                  if ((__cps_v_191[LUMO_TAG] === "true")) {
                    return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_190) => {
                      if ((__cps_v_190[LUMO_TAG] === "less")) {
                        return __k_112(Bool["true"]);
                      } else if ((__cps_v_190[LUMO_TAG] === "equal")) {
                        return __k_112(Bool["true"]);
                      } else {
                        return ((__cps_v_190[LUMO_TAG] === "greater") ? __k_112(Bool["false"]) : __lumo_match_error(__cps_v_190));
                      }
                    });
                  } else if ((__cps_v_191[LUMO_TAG] === "false")) {
                    return __k_112(Bool["false"]);
                  } else {
                    return __lumo_match_error(__cps_v_191);
                  }
                };
                if ((__cps_v_192[LUMO_TAG] === "less")) {
                  return __k_114(Bool["false"]);
                } else if ((__cps_v_192[LUMO_TAG] === "equal")) {
                  return __k_114(Bool["true"]);
                } else {
                  return ((__cps_v_192[LUMO_TAG] === "greater") ? __k_114(Bool["true"]) : __lumo_match_error(__cps_v_192));
                }
              });
            });
          });
        } else {
          return __lumo_match_error(__cps_v_193);
        }
      };
      if ((__cps_v_194[LUMO_TAG] === "less")) {
        return __k_115(Bool["false"]);
      } else if ((__cps_v_194[LUMO_TAG] === "equal")) {
        return __k_115(Bool["true"]);
      } else {
        return ((__cps_v_194[LUMO_TAG] === "greater") ? __k_115(Bool["true"]) : __lumo_match_error(__cps_v_194));
      }
    });
  });
}

export function to_upper_char(__caps, c, __k) {
  return String.char_code_at(__caps, c, 0, (code) => {
    return __caps.PartialOrd_Number.cmp(__caps, code, 97, (__cps_v_200) => {
      const __k_119 = (__cps_v_199) => {
        if ((__cps_v_199[LUMO_TAG] === "true")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 122, (__cps_v_198) => {
            const __k_118 = (__cps_v_197) => {
              if ((__cps_v_197[LUMO_TAG] === "true")) {
                return __caps.Sub_Number.sub(__caps, code, 32, (__cps_v_196) => {
                  return __caps.StrOps_StrOps.from_char_code(__caps, __cps_v_196, __k);
                });
              } else if ((__cps_v_197[LUMO_TAG] === "false")) {
                return __k(c);
              } else {
                return __lumo_match_error(__cps_v_197);
              }
            };
            if ((__cps_v_198[LUMO_TAG] === "less")) {
              return __k_118(Bool["true"]);
            } else if ((__cps_v_198[LUMO_TAG] === "equal")) {
              return __k_118(Bool["true"]);
            } else {
              return ((__cps_v_198[LUMO_TAG] === "greater") ? __k_118(Bool["false"]) : __lumo_match_error(__cps_v_198));
            }
          });
        } else if ((__cps_v_199[LUMO_TAG] === "false")) {
          return __k(c);
        } else {
          return __lumo_match_error(__cps_v_199);
        }
      };
      if ((__cps_v_200[LUMO_TAG] === "less")) {
        return __k_119(Bool["false"]);
      } else if ((__cps_v_200[LUMO_TAG] === "equal")) {
        return __k_119(Bool["true"]);
      } else {
        return ((__cps_v_200[LUMO_TAG] === "greater") ? __k_119(Bool["true"]) : __lumo_match_error(__cps_v_200));
      }
    });
  });
}

export function keyword_variant(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__cps_v_201) => {
    return __caps.Add_String.add(__caps, __cps_v_201, "_KW", __k);
  });
}

export function to_upper_string(__caps, s, __k) {
  return to_upper_string_loop(__caps, s, 0, "", __k);
}

export function to_upper_string_loop(__caps, s, i, acc, __k) {
  return String.len(__caps, s, (__cps_v_208) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_208, (__cps_v_207) => {
      const __k_121 = (__cps_v_206) => {
        if ((__cps_v_206[LUMO_TAG] === "true")) {
          return __k(acc);
        } else if ((__cps_v_206[LUMO_TAG] === "false")) {
          return __caps.Add_Number.add(__caps, i, 1, (__cps_v_202) => {
            return String.char_at(__caps, s, i, (__cps_v_205) => {
              return to_upper_char(__caps, __cps_v_205, (__cps_v_204) => {
                return __caps.Add_String.add(__caps, acc, __cps_v_204, (__cps_v_203) => {
                  return to_upper_string_loop(__caps, s, __cps_v_202, __cps_v_203, __k);
                });
              });
            });
          });
        } else {
          return __lumo_match_error(__cps_v_206);
        }
      };
      if ((__cps_v_207[LUMO_TAG] === "less")) {
        return __k_121(Bool["false"]);
      } else if ((__cps_v_207[LUMO_TAG] === "equal")) {
        return __k_121(Bool["true"]);
      } else {
        return ((__cps_v_207[LUMO_TAG] === "greater") ? __k_121(Bool["true"]) : __lumo_match_error(__cps_v_207));
      }
    });
  });
}

export function symbol_variant(__caps, sym, __k) {
  return __thunk(() => {
    return __caps.PartialEq_String.eq(__caps, sym, "#", (__cps_v_238) => {
      if ((__cps_v_238[LUMO_TAG] === "true")) {
        return __k("HASH");
      } else if ((__cps_v_238[LUMO_TAG] === "false")) {
        return __caps.PartialEq_String.eq(__caps, sym, "(", (__cps_v_237) => {
          if ((__cps_v_237[LUMO_TAG] === "true")) {
            return __k("L_PAREN");
          } else if ((__cps_v_237[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, sym, ")", (__cps_v_236) => {
              if ((__cps_v_236[LUMO_TAG] === "true")) {
                return __k("R_PAREN");
              } else if ((__cps_v_236[LUMO_TAG] === "false")) {
                return __caps.PartialEq_String.eq(__caps, sym, "[", (__cps_v_235) => {
                  if ((__cps_v_235[LUMO_TAG] === "true")) {
                    return __k("L_BRACKET");
                  } else if ((__cps_v_235[LUMO_TAG] === "false")) {
                    return __caps.PartialEq_String.eq(__caps, sym, "]", (__cps_v_234) => {
                      if ((__cps_v_234[LUMO_TAG] === "true")) {
                        return __k("R_BRACKET");
                      } else if ((__cps_v_234[LUMO_TAG] === "false")) {
                        return __caps.PartialEq_String.eq(__caps, sym, "{", (__cps_v_233) => {
                          if ((__cps_v_233[LUMO_TAG] === "true")) {
                            return __k("L_BRACE");
                          } else if ((__cps_v_233[LUMO_TAG] === "false")) {
                            return __caps.PartialEq_String.eq(__caps, sym, "}", (__cps_v_232) => {
                              if ((__cps_v_232[LUMO_TAG] === "true")) {
                                return __k("R_BRACE");
                              } else if ((__cps_v_232[LUMO_TAG] === "false")) {
                                return __caps.PartialEq_String.eq(__caps, sym, ";", (__cps_v_231) => {
                                  if ((__cps_v_231[LUMO_TAG] === "true")) {
                                    return __k("SEMICOLON");
                                  } else if ((__cps_v_231[LUMO_TAG] === "false")) {
                                    return __caps.PartialEq_String.eq(__caps, sym, ":", (__cps_v_230) => {
                                      if ((__cps_v_230[LUMO_TAG] === "true")) {
                                        return __k("COLON");
                                      } else if ((__cps_v_230[LUMO_TAG] === "false")) {
                                        return __caps.PartialEq_String.eq(__caps, sym, ",", (__cps_v_229) => {
                                          if ((__cps_v_229[LUMO_TAG] === "true")) {
                                            return __k("COMMA");
                                          } else if ((__cps_v_229[LUMO_TAG] === "false")) {
                                            return __caps.PartialEq_String.eq(__caps, sym, "=", (__cps_v_228) => {
                                              if ((__cps_v_228[LUMO_TAG] === "true")) {
                                                return __k("EQUALS");
                                              } else if ((__cps_v_228[LUMO_TAG] === "false")) {
                                                return __caps.PartialEq_String.eq(__caps, sym, ":=", (__cps_v_227) => {
                                                  if ((__cps_v_227[LUMO_TAG] === "true")) {
                                                    return __k("COLON_EQ");
                                                  } else if ((__cps_v_227[LUMO_TAG] === "false")) {
                                                    return __caps.PartialEq_String.eq(__caps, sym, "=>", (__cps_v_226) => {
                                                      if ((__cps_v_226[LUMO_TAG] === "true")) {
                                                        return __k("FAT_ARROW");
                                                      } else if ((__cps_v_226[LUMO_TAG] === "false")) {
                                                        return __caps.PartialEq_String.eq(__caps, sym, "->", (__cps_v_225) => {
                                                          if ((__cps_v_225[LUMO_TAG] === "true")) {
                                                            return __k("ARROW");
                                                          } else if ((__cps_v_225[LUMO_TAG] === "false")) {
                                                            return __caps.PartialEq_String.eq(__caps, sym, ".", (__cps_v_224) => {
                                                              if ((__cps_v_224[LUMO_TAG] === "true")) {
                                                                return __k("DOT");
                                                              } else if ((__cps_v_224[LUMO_TAG] === "false")) {
                                                                return __caps.PartialEq_String.eq(__caps, sym, "+", (__cps_v_223) => {
                                                                  if ((__cps_v_223[LUMO_TAG] === "true")) {
                                                                    return __k("PLUS");
                                                                  } else if ((__cps_v_223[LUMO_TAG] === "false")) {
                                                                    return __caps.PartialEq_String.eq(__caps, sym, "-", (__cps_v_222) => {
                                                                      if ((__cps_v_222[LUMO_TAG] === "true")) {
                                                                        return __k("MINUS");
                                                                      } else if ((__cps_v_222[LUMO_TAG] === "false")) {
                                                                        return __caps.PartialEq_String.eq(__caps, sym, "*", (__cps_v_221) => {
                                                                          if ((__cps_v_221[LUMO_TAG] === "true")) {
                                                                            return __k("STAR");
                                                                          } else if ((__cps_v_221[LUMO_TAG] === "false")) {
                                                                            return __caps.PartialEq_String.eq(__caps, sym, "/", (__cps_v_220) => {
                                                                              if ((__cps_v_220[LUMO_TAG] === "true")) {
                                                                                return __k("SLASH");
                                                                              } else if ((__cps_v_220[LUMO_TAG] === "false")) {
                                                                                return __caps.PartialEq_String.eq(__caps, sym, "%", (__cps_v_219) => {
                                                                                  if ((__cps_v_219[LUMO_TAG] === "true")) {
                                                                                    return __k("PERCENT");
                                                                                  } else if ((__cps_v_219[LUMO_TAG] === "false")) {
                                                                                    return __caps.PartialEq_String.eq(__caps, sym, "!", (__cps_v_218) => {
                                                                                      if ((__cps_v_218[LUMO_TAG] === "true")) {
                                                                                        return __k("BANG");
                                                                                      } else if ((__cps_v_218[LUMO_TAG] === "false")) {
                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "<", (__cps_v_217) => {
                                                                                          if ((__cps_v_217[LUMO_TAG] === "true")) {
                                                                                            return __k("LT");
                                                                                          } else if ((__cps_v_217[LUMO_TAG] === "false")) {
                                                                                            return __caps.PartialEq_String.eq(__caps, sym, ">", (__cps_v_216) => {
                                                                                              if ((__cps_v_216[LUMO_TAG] === "true")) {
                                                                                                return __k("GT");
                                                                                              } else if ((__cps_v_216[LUMO_TAG] === "false")) {
                                                                                                return __caps.PartialEq_String.eq(__caps, sym, "<=", (__cps_v_215) => {
                                                                                                  if ((__cps_v_215[LUMO_TAG] === "true")) {
                                                                                                    return __k("LT_EQ");
                                                                                                  } else if ((__cps_v_215[LUMO_TAG] === "false")) {
                                                                                                    return __caps.PartialEq_String.eq(__caps, sym, ">=", (__cps_v_214) => {
                                                                                                      if ((__cps_v_214[LUMO_TAG] === "true")) {
                                                                                                        return __k("GT_EQ");
                                                                                                      } else if ((__cps_v_214[LUMO_TAG] === "false")) {
                                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "==", (__cps_v_213) => {
                                                                                                          if ((__cps_v_213[LUMO_TAG] === "true")) {
                                                                                                            return __k("EQ_EQ");
                                                                                                          } else if ((__cps_v_213[LUMO_TAG] === "false")) {
                                                                                                            return __caps.PartialEq_String.eq(__caps, sym, "!=", (__cps_v_212) => {
                                                                                                              if ((__cps_v_212[LUMO_TAG] === "true")) {
                                                                                                                return __k("BANG_EQ");
                                                                                                              } else if ((__cps_v_212[LUMO_TAG] === "false")) {
                                                                                                                return __caps.PartialEq_String.eq(__caps, sym, "&&", (__cps_v_211) => {
                                                                                                                  if ((__cps_v_211[LUMO_TAG] === "true")) {
                                                                                                                    return __k("AMP_AMP");
                                                                                                                  } else if ((__cps_v_211[LUMO_TAG] === "false")) {
                                                                                                                    return __caps.PartialEq_String.eq(__caps, sym, "||", (__cps_v_210) => {
                                                                                                                      if ((__cps_v_210[LUMO_TAG] === "true")) {
                                                                                                                        return __k("PIPE_PIPE");
                                                                                                                      } else if ((__cps_v_210[LUMO_TAG] === "false")) {
                                                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "_", (__cps_v_209) => {
                                                                                                                          if ((__cps_v_209[LUMO_TAG] === "true")) {
                                                                                                                            return __k("UNDERSCORE");
                                                                                                                          } else if ((__cps_v_209[LUMO_TAG] === "false")) {
                                                                                                                            return __caps.Add_String.add(__caps, "SYM_", sym, __k);
                                                                                                                          } else {
                                                                                                                            return __lumo_match_error(__cps_v_209);
                                                                                                                          }
                                                                                                                        });
                                                                                                                      } else {
                                                                                                                        return __lumo_match_error(__cps_v_210);
                                                                                                                      }
                                                                                                                    });
                                                                                                                  } else {
                                                                                                                    return __lumo_match_error(__cps_v_211);
                                                                                                                  }
                                                                                                                });
                                                                                                              } else {
                                                                                                                return __lumo_match_error(__cps_v_212);
                                                                                                              }
                                                                                                            });
                                                                                                          } else {
                                                                                                            return __lumo_match_error(__cps_v_213);
                                                                                                          }
                                                                                                        });
                                                                                                      } else {
                                                                                                        return __lumo_match_error(__cps_v_214);
                                                                                                      }
                                                                                                    });
                                                                                                  } else {
                                                                                                    return __lumo_match_error(__cps_v_215);
                                                                                                  }
                                                                                                });
                                                                                              } else {
                                                                                                return __lumo_match_error(__cps_v_216);
                                                                                              }
                                                                                            });
                                                                                          } else {
                                                                                            return __lumo_match_error(__cps_v_217);
                                                                                          }
                                                                                        });
                                                                                      } else {
                                                                                        return __lumo_match_error(__cps_v_218);
                                                                                      }
                                                                                    });
                                                                                  } else {
                                                                                    return __lumo_match_error(__cps_v_219);
                                                                                  }
                                                                                });
                                                                              } else {
                                                                                return __lumo_match_error(__cps_v_220);
                                                                              }
                                                                            });
                                                                          } else {
                                                                            return __lumo_match_error(__cps_v_221);
                                                                          }
                                                                        });
                                                                      } else {
                                                                        return __lumo_match_error(__cps_v_222);
                                                                      }
                                                                    });
                                                                  } else {
                                                                    return __lumo_match_error(__cps_v_223);
                                                                  }
                                                                });
                                                              } else {
                                                                return __lumo_match_error(__cps_v_224);
                                                              }
                                                            });
                                                          } else {
                                                            return __lumo_match_error(__cps_v_225);
                                                          }
                                                        });
                                                      } else {
                                                        return __lumo_match_error(__cps_v_226);
                                                      }
                                                    });
                                                  } else {
                                                    return __lumo_match_error(__cps_v_227);
                                                  }
                                                });
                                              } else {
                                                return __lumo_match_error(__cps_v_228);
                                              }
                                            });
                                          } else {
                                            return __lumo_match_error(__cps_v_229);
                                          }
                                        });
                                      } else {
                                        return __lumo_match_error(__cps_v_230);
                                      }
                                    });
                                  } else {
                                    return __lumo_match_error(__cps_v_231);
                                  }
                                });
                              } else {
                                return __lumo_match_error(__cps_v_232);
                              }
                            });
                          } else {
                            return __lumo_match_error(__cps_v_233);
                          }
                        });
                      } else {
                        return __lumo_match_error(__cps_v_234);
                      }
                    });
                  } else {
                    return __lumo_match_error(__cps_v_235);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_236);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_237);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_238);
      }
    });
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
          return dedupe_strings(__caps, kws, (__cps_v_242) => {
            return sort_strings(__caps, __cps_v_242, (__cps_v_239) => {
              return dedupe_strings(__caps, syms, (__cps_v_241) => {
                return sort_strings(__caps, __cps_v_241, (__cps_v_240) => {
                  return __k(CollectedTokens["mk"](__cps_v_239, __cps_v_240));
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
      return collect_tokens_from_alts(__caps, alts, kws, syms, __k);
    } else {
      return __lumo_match_error(body);
    }
  });
}

export function collect_tokens_from_alts(__caps, alts, kws, syms, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return String.char_code_at(__caps, name, 0, (code) => {
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_246) => {
            const __k_163 = (__cps_v_245) => {
              if ((__cps_v_245[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_244) => {
                  const __k_162 = (__cps_v_243) => {
                    if ((__cps_v_243[LUMO_TAG] === "true")) {
                      return collect_tokens_from_alts(__caps, rest, kws, syms, __k);
                    } else if ((__cps_v_243[LUMO_TAG] === "false")) {
                      return collect_alt_token(__caps, name, rest, kws, syms, __k);
                    } else {
                      return __lumo_match_error(__cps_v_243);
                    }
                  };
                  if ((__cps_v_244[LUMO_TAG] === "less")) {
                    return __k_162(Bool["true"]);
                  } else if ((__cps_v_244[LUMO_TAG] === "equal")) {
                    return __k_162(Bool["true"]);
                  } else {
                    return ((__cps_v_244[LUMO_TAG] === "greater") ? __k_162(Bool["false"]) : __lumo_match_error(__cps_v_244));
                  }
                });
              } else if ((__cps_v_245[LUMO_TAG] === "false")) {
                return collect_alt_token(__caps, name, rest, kws, syms, __k);
              } else {
                return __lumo_match_error(__cps_v_245);
              }
            };
            if ((__cps_v_246[LUMO_TAG] === "less")) {
              return __k_163(Bool["false"]);
            } else if ((__cps_v_246[LUMO_TAG] === "equal")) {
              return __k_163(Bool["true"]);
            } else {
              return ((__cps_v_246[LUMO_TAG] === "greater") ? __k_163(Bool["true"]) : __lumo_match_error(__cps_v_246));
            }
          });
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function collect_alt_token(__caps, name, rest, kws, syms, __k) {
  return has_alpha(__caps, name, 0, (__cps_v_247) => {
    if ((__cps_v_247[LUMO_TAG] === "true")) {
      return collect_tokens_from_alts(__caps, rest, List["cons"](name, kws), syms, __k);
    } else if ((__cps_v_247[LUMO_TAG] === "false")) {
      return collect_tokens_from_alts(__caps, rest, kws, List["cons"](name, syms), __k);
    } else {
      return __lumo_match_error(__cps_v_247);
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
      return list_contains_string(__caps, acc, x, (__cps_v_248) => {
        if ((__cps_v_248[LUMO_TAG] === "true")) {
          return dedupe_strings_acc(__caps, rest, acc, __k);
        } else if ((__cps_v_248[LUMO_TAG] === "false")) {
          return dedupe_strings_acc(__caps, rest, List["cons"](x, acc), __k);
        } else {
          return __lumo_match_error(__cps_v_248);
        }
      });
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
      return insert_sorted(__caps, x, sorted, (__cps_v_249) => {
        return sort_strings_acc(__caps, rest, __cps_v_249, __k);
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
      return string_lt(__caps, s, x, (__cps_v_251) => {
        if ((__cps_v_251[LUMO_TAG] === "true")) {
          return __k(List["cons"](s, xs));
        } else if ((__cps_v_251[LUMO_TAG] === "false")) {
          return insert_sorted(__caps, s, rest, (__cps_v_250) => {
            return __k(List["cons"](x, __cps_v_250));
          });
        } else {
          return __lumo_match_error(__cps_v_251);
        }
      });
    } else {
      return __lumo_match_error(xs);
    }
  });
}

export function string_lt(__caps, a, b, __k) {
  return string_lt_loop(__caps, a, b, 0, __k);
}

export function string_lt_loop(__caps, a, b, i, __k) {
  return String.len(__caps, a, (__cps_v_265) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_265, (__cps_v_264) => {
      const __k_179 = (__cps_v_263) => {
        if ((__cps_v_263[LUMO_TAG] === "true")) {
          return String.len(__caps, b, (__cps_v_262) => {
            return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_262, (__cps_v_261) => {
              const __k_178 = (__cps_v_260) => {
                if ((__cps_v_260[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((__cps_v_260[LUMO_TAG] === "false")) {
                  return __k(Bool["true"]);
                } else {
                  return __lumo_match_error(__cps_v_260);
                }
              };
              if ((__cps_v_261[LUMO_TAG] === "less")) {
                return __k_178(Bool["false"]);
              } else if ((__cps_v_261[LUMO_TAG] === "equal")) {
                return __k_178(Bool["true"]);
              } else {
                return ((__cps_v_261[LUMO_TAG] === "greater") ? __k_178(Bool["true"]) : __lumo_match_error(__cps_v_261));
              }
            });
          });
        } else if ((__cps_v_263[LUMO_TAG] === "false")) {
          return String.len(__caps, b, (__cps_v_259) => {
            return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_259, (__cps_v_258) => {
              const __k_176 = (__cps_v_257) => {
                if ((__cps_v_257[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((__cps_v_257[LUMO_TAG] === "false")) {
                  return String.char_code_at(__caps, a, i, (ca) => {
                    return String.char_code_at(__caps, b, i, (cb) => {
                      return __caps.PartialOrd_Number.cmp(__caps, ca, cb, (__cps_v_256) => {
                        const __k_175 = (__cps_v_255) => {
                          if ((__cps_v_255[LUMO_TAG] === "true")) {
                            return __k(Bool["true"]);
                          } else if ((__cps_v_255[LUMO_TAG] === "false")) {
                            return __caps.PartialOrd_Number.cmp(__caps, cb, ca, (__cps_v_254) => {
                              const __k_174 = (__cps_v_253) => {
                                if ((__cps_v_253[LUMO_TAG] === "true")) {
                                  return __k(Bool["false"]);
                                } else if ((__cps_v_253[LUMO_TAG] === "false")) {
                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_252) => {
                                    return string_lt_loop(__caps, a, b, __cps_v_252, __k);
                                  });
                                } else {
                                  return __lumo_match_error(__cps_v_253);
                                }
                              };
                              if ((__cps_v_254[LUMO_TAG] === "less")) {
                                return __k_174(Bool["true"]);
                              } else if ((__cps_v_254[LUMO_TAG] === "equal")) {
                                return __k_174(Bool["false"]);
                              } else {
                                return ((__cps_v_254[LUMO_TAG] === "greater") ? __k_174(Bool["false"]) : __lumo_match_error(__cps_v_254));
                              }
                            });
                          } else {
                            return __lumo_match_error(__cps_v_255);
                          }
                        };
                        if ((__cps_v_256[LUMO_TAG] === "less")) {
                          return __k_175(Bool["true"]);
                        } else if ((__cps_v_256[LUMO_TAG] === "equal")) {
                          return __k_175(Bool["false"]);
                        } else {
                          return ((__cps_v_256[LUMO_TAG] === "greater") ? __k_175(Bool["false"]) : __lumo_match_error(__cps_v_256));
                        }
                      });
                    });
                  });
                } else {
                  return __lumo_match_error(__cps_v_257);
                }
              };
              if ((__cps_v_258[LUMO_TAG] === "less")) {
                return __k_176(Bool["false"]);
              } else if ((__cps_v_258[LUMO_TAG] === "equal")) {
                return __k_176(Bool["true"]);
              } else {
                return ((__cps_v_258[LUMO_TAG] === "greater") ? __k_176(Bool["true"]) : __lumo_match_error(__cps_v_258));
              }
            });
          });
        } else {
          return __lumo_match_error(__cps_v_263);
        }
      };
      if ((__cps_v_264[LUMO_TAG] === "less")) {
        return __k_179(Bool["false"]);
      } else if ((__cps_v_264[LUMO_TAG] === "equal")) {
        return __k_179(Bool["true"]);
      } else {
        return ((__cps_v_264[LUMO_TAG] === "greater") ? __k_179(Bool["true"]) : __lumo_match_error(__cps_v_264));
      }
    });
  });
}

export function is_token_only_alternatives(__caps, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(Bool["true"]);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return String.char_code_at(__caps, name, 0, (code) => {
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_268) => {
            const __k_185 = (__cps_v_267) => {
              const __k_183 = (is_upper) => {
                if ((is_upper[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((is_upper[LUMO_TAG] === "false")) {
                  return is_token_only_alternatives(__caps, rest, __k);
                } else {
                  return __lumo_match_error(is_upper);
                }
              };
              if ((__cps_v_267[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_266) => {
                  if ((__cps_v_266[LUMO_TAG] === "less")) {
                    return __k_183(Bool["true"]);
                  } else if ((__cps_v_266[LUMO_TAG] === "equal")) {
                    return __k_183(Bool["true"]);
                  } else {
                    return ((__cps_v_266[LUMO_TAG] === "greater") ? __k_183(Bool["false"]) : __lumo_match_error(__cps_v_266));
                  }
                });
              } else if ((__cps_v_267[LUMO_TAG] === "false")) {
                return __k_183(Bool["false"]);
              } else {
                return __lumo_match_error(__cps_v_267);
              }
            };
            if ((__cps_v_268[LUMO_TAG] === "less")) {
              return __k_185(Bool["false"]);
            } else if ((__cps_v_268[LUMO_TAG] === "equal")) {
              return __k_185(Bool["true"]);
            } else {
              return ((__cps_v_268[LUMO_TAG] === "greater") ? __k_185(Bool["true"]) : __lumo_match_error(__cps_v_268));
            }
          });
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function generate_syntax_kind(__caps, grammar, __k) {
  return collect_tokens(__caps, grammar, (collected) => {
    if ((collected[LUMO_TAG] === "mk")) {
      const keywords = collected.args[0];
      const symbols = collected.args[1];
      if ((grammar[LUMO_TAG] === "mk")) {
        const token_defs = grammar.args[0];
        const rules = grammar.args[1];
        const s = "// Auto-generated by langue. Do not edit.\n";
        return __caps.Add_String.add(__caps, s, "// Regenerate: scripts/gen_langue.sh\n\n", (s) => {
          return __caps.Add_String.add(__caps, s, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n", (s) => {
            return __caps.Add_String.add(__caps, s, "#[repr(u16)]\n", (s) => {
              return __caps.Add_String.add(__caps, s, "pub enum SyntaxKind {\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    // Named tokens\n", (s) => {
                  return emit_named_tokens(__caps, s, token_defs, (s) => {
                    return __caps.Add_String.add(__caps, s, "    // Trivia\n", (s) => {
                      return __caps.Add_String.add(__caps, s, "    WHITESPACE,\n    NEWLINE,\n    UNKNOWN,\n", (s) => {
                        return emit_keywords(__caps, s, keywords, (s) => {
                          return emit_symbols(__caps, s, symbols, (s) => {
                            return __caps.Add_String.add(__caps, s, "    // Nodes\n", (s) => {
                              return emit_node_kinds(__caps, s, rules, (s) => {
                                return __caps.Add_String.add(__caps, s, "    // Sentinel\n    ERROR,\n", (s) => {
                                  return __caps.Add_String.add(__caps, s, "}\n", (s) => {
                                    return __caps.Add_String.add(__caps, s, "\nimpl SyntaxKind {\n", (s) => {
                                      return __caps.Add_String.add(__caps, s, "    pub fn is_trivia(self) -> bool {\n", (s) => {
                                        return __caps.Add_String.add(__caps, s, "        matches!(self, Self::WHITESPACE | Self::NEWLINE)\n", (s) => {
                                          return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                                            return emit_from_keyword(__caps, s, keywords, (s) => {
                                              return emit_from_symbol(__caps, s, symbols, (s) => {
                                                return __caps.Add_String.add(__caps, s, "}\n", (s) => {
                                                  return __k(s);
                                                });
                                              });
                                            });
                                          });
                                        });
                                      });
                                    });
                                  });
                                });
                              });
                            });
                          });
                        });
                      });
                    });
                  });
                });
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

export function emit_named_tokens(__caps, s, tokens, __k) {
  return __thunk(() => {
    if ((tokens[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((tokens[LUMO_TAG] === "cons")) {
      const tok = tokens.args[0];
      const rest = tokens.args[1];
      return __caps.Add_String.add(__caps, s, "    ", (__cps_v_271) => {
        return to_screaming_snake(__caps, tok, (__cps_v_272) => {
          return __caps.Add_String.add(__caps, __cps_v_271, __cps_v_272, (__cps_v_270) => {
            return __caps.Add_String.add(__caps, __cps_v_270, ",\n", (__cps_v_269) => {
              return emit_named_tokens(__caps, __cps_v_269, rest, __k);
            });
          });
        });
      });
    } else {
      return __lumo_match_error(tokens);
    }
  });
}

export function emit_keywords(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return __caps.Add_String.add(__caps, s, "    // Keywords\n", (s2) => {
        return emit_keywords_items(__caps, s2, kws, __k);
      });
    }
  });
}

export function emit_keywords_items(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      return keyword_variant(__caps, kw, (__cps_v_277) => {
        return __caps.Add_String.add(__caps, "    ", __cps_v_277, (__cps_v_276) => {
          return __caps.Add_String.add(__caps, __cps_v_276, ", // '", (__cps_v_275) => {
            return __caps.Add_String.add(__caps, __cps_v_275, kw, (__cps_v_274) => {
              return __caps.Add_String.add(__caps, __cps_v_274, "'\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_273) => {
                  return emit_keywords_items(__caps, __cps_v_273, rest, __k);
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_symbols(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      const __k_192 = (s2) => {
        return emit_symbols_items(__caps, s2, syms, __k);
      };
      if ((rest[LUMO_TAG] === "nil")) {
        return __k_192(s);
      } else {
        return __caps.Add_String.add(__caps, s, "    // Symbols\n", __k_192);
      }
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function emit_symbols_items(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      return symbol_variant(__caps, sym, (__cps_v_282) => {
        return __caps.Add_String.add(__caps, "    ", __cps_v_282, (__cps_v_281) => {
          return __caps.Add_String.add(__caps, __cps_v_281, ", // '", (__cps_v_280) => {
            return __caps.Add_String.add(__caps, __cps_v_280, sym, (__cps_v_279) => {
              return __caps.Add_String.add(__caps, __cps_v_279, "'\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_278) => {
                  return emit_symbols_items(__caps, __cps_v_278, rest, __k);
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function emit_node_kinds(__caps, s, rules, __k) {
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
          return to_screaming_snake(__caps, name, (__cps_v_293) => {
            return __caps.Add_String.add(__caps, "    ", __cps_v_293, (__cps_v_292) => {
              return __caps.Add_String.add(__caps, __cps_v_292, ", // ", (__cps_v_291) => {
                return __caps.Add_String.add(__caps, __cps_v_291, name, (__cps_v_290) => {
                  return __caps.Add_String.add(__caps, __cps_v_290, "\n", (line) => {
                    return __caps.Add_String.add(__caps, s, line, (__cps_v_289) => {
                      return emit_node_kinds(__caps, __cps_v_289, rest, __k);
                    });
                  });
                });
              });
            });
          });
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          return is_token_only_alternatives(__caps, alts, (__cps_v_288) => {
            if ((__cps_v_288[LUMO_TAG] === "true")) {
              return to_screaming_snake(__caps, name, (__cps_v_287) => {
                return __caps.Add_String.add(__caps, "    ", __cps_v_287, (__cps_v_286) => {
                  return __caps.Add_String.add(__caps, __cps_v_286, ", // ", (__cps_v_285) => {
                    return __caps.Add_String.add(__caps, __cps_v_285, name, (__cps_v_284) => {
                      return __caps.Add_String.add(__caps, __cps_v_284, " (token wrapper)\n", (line) => {
                        return __caps.Add_String.add(__caps, s, line, (__cps_v_283) => {
                          return emit_node_kinds(__caps, __cps_v_283, rest, __k);
                        });
                      });
                    });
                  });
                });
              });
            } else if ((__cps_v_288[LUMO_TAG] === "false")) {
              return emit_node_kinds(__caps, s, rest, __k);
            } else {
              return __lumo_match_error(__cps_v_288);
            }
          });
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

export function emit_from_keyword(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return __caps.Add_String.add(__caps, s, "\n    pub fn from_keyword(text: &str) -> Option<Self> {\n", (s) => {
        return __caps.Add_String.add(__caps, s, "        match text {\n", (s) => {
          return emit_keyword_arms(__caps, s, kws, (s) => {
            return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
              return __caps.Add_String.add(__caps, s, "        }\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                  return __k(s);
                });
              });
            });
          });
        });
      });
    }
  });
}

export function emit_keyword_arms(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      return __caps.Add_String.add(__caps, "            \"", kw, (__cps_v_298) => {
        return __caps.Add_String.add(__caps, __cps_v_298, "\" => Some(Self::", (__cps_v_296) => {
          return keyword_variant(__caps, kw, (__cps_v_297) => {
            return __caps.Add_String.add(__caps, __cps_v_296, __cps_v_297, (__cps_v_295) => {
              return __caps.Add_String.add(__caps, __cps_v_295, "),\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_294) => {
                  return emit_keyword_arms(__caps, __cps_v_294, rest, __k);
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_from_symbol(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      return __caps.Add_String.add(__caps, s, "\n    pub fn from_symbol(text: &str) -> Option<Self> {\n", (s) => {
        return __caps.Add_String.add(__caps, s, "        match text {\n", (s) => {
          return emit_symbol_arms(__caps, s, syms, (s) => {
            return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
              return __caps.Add_String.add(__caps, s, "        }\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                  return __k(s);
                });
              });
            });
          });
        });
      });
    }
  });
}

export function emit_symbol_arms(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      return __caps.Add_String.add(__caps, "            \"", sym, (__cps_v_303) => {
        return __caps.Add_String.add(__caps, __cps_v_303, "\" => Some(Self::", (__cps_v_301) => {
          return symbol_variant(__caps, sym, (__cps_v_302) => {
            return __caps.Add_String.add(__caps, __cps_v_301, __cps_v_302, (__cps_v_300) => {
              return __caps.Add_String.add(__caps, __cps_v_300, "),\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_299) => {
                  return emit_symbol_arms(__caps, __cps_v_299, rest, __k);
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function generate_ast(__caps, grammar, __k) {
  return __thunk(() => {
    if ((grammar[LUMO_TAG] === "mk")) {
      const token_defs = grammar.args[0];
      const rules = grammar.args[1];
      const s = "// Auto-generated by langue. Do not edit.\n";
      return __caps.Add_String.add(__caps, s, "// Regenerate: scripts/gen_langue.sh\n\n", (s) => {
        return __caps.Add_String.add(__caps, s, "use super::SyntaxKind;\n", (s) => {
          return __caps.Add_String.add(__caps, s, "use super::{SyntaxNode, SyntaxElement, LosslessToken};\n\n", (s) => {
            return __caps.Add_String.add(__caps, s, "pub trait AstNode<'a>: Sized {\n", (s) => {
              return __caps.Add_String.add(__caps, s, "    fn cast(node: &'a SyntaxNode) -> Option<Self>;\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    fn syntax(&self) -> &'a SyntaxNode;\n", (s) => {
                  return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
                    return emit_ast_rules(__caps, s, token_defs, rules, __k);
                  });
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(grammar);
    }
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
        const __k_206 = (s2) => {
          return emit_ast_rules(__caps, s2, token_defs, rest, __k);
        };
        if ((body[LUMO_TAG] === "sequence")) {
          const elems = body.args[0];
          return emit_struct_node(__caps, s, name, elems, token_defs, __k_206);
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          return is_token_only_alternatives(__caps, alts, (__cps_v_304) => {
            if ((__cps_v_304[LUMO_TAG] === "true")) {
              return emit_token_wrapper_node(__caps, s, name, __k_206);
            } else if ((__cps_v_304[LUMO_TAG] === "false")) {
              return emit_enum_node(__caps, s, name, alts, __k_206);
            } else {
              return __lumo_match_error(__cps_v_304);
            }
          });
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

export function emit_struct_node(__caps, s, name, elems, token_defs, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return __caps.Add_String.add(__caps, s, "pub struct ", (__cps_v_310) => {
      return __caps.Add_String.add(__caps, __cps_v_310, name, (__cps_v_309) => {
        return __caps.Add_String.add(__caps, __cps_v_309, "<'a>(pub(crate) &'a SyntaxNode);\n\n", (s) => {
          return __caps.Add_String.add(__caps, s, "impl<'a> AstNode<'a> for ", (__cps_v_308) => {
            return __caps.Add_String.add(__caps, __cps_v_308, name, (__cps_v_307) => {
              return __caps.Add_String.add(__caps, __cps_v_307, "<'a> {\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n", (s) => {
                  return __caps.Add_String.add(__caps, s, "        (node.kind == SyntaxKind::", (__cps_v_306) => {
                    return __caps.Add_String.add(__caps, __cps_v_306, kind, (__cps_v_305) => {
                      return __caps.Add_String.add(__caps, __cps_v_305, ").then(|| Self(node))\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                          return __caps.Add_String.add(__caps, s, "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n", (s) => {
                            return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
                              return emit_accessors(__caps, s, name, elems, token_defs, __k);
                            });
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    });
  });
}

export function emit_accessors(__caps, s, struct_name, elems, token_defs, __k) {
  return __thunk(() => {
    const has_labeled = has_labeled_elements(elems);
    if ((has_labeled[LUMO_TAG] === "true")) {
      return __caps.Add_String.add(__caps, s, "impl<'a> ", (__cps_v_312) => {
        return __caps.Add_String.add(__caps, __cps_v_312, struct_name, (__cps_v_311) => {
          return __caps.Add_String.add(__caps, __cps_v_311, "<'a> {\n", (s) => {
            return emit_accessors_for_elements(__caps, s, elems, token_defs, (s) => {
              return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
                return __k(s);
              });
            });
          });
        });
      });
    } else if ((has_labeled[LUMO_TAG] === "false")) {
      return __k(s);
    } else {
      return __lumo_match_error(has_labeled);
    }
  });
}

export function has_labeled_elements(elems) {
  if ((elems[LUMO_TAG] === "nil")) {
    return Bool["false"];
  } else if ((elems[LUMO_TAG] === "cons")) {
    const elem = elems.args[0];
    const rest = elems.args[1];
    if ((elem[LUMO_TAG] === "labeled")) {
      return Bool["true"];
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
      return emit_token_accessor(__caps, s, label, t, Bool["false"], __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        return list_contains_string(__caps, token_defs, name, (__cps_v_313) => {
          if ((__cps_v_313[LUMO_TAG] === "true")) {
            return emit_token_accessor(__caps, s, label, TokenRef["named"](name), Bool["false"], __k);
          } else if ((__cps_v_313[LUMO_TAG] === "false")) {
            return emit_node_accessor(__caps, s, label, name, Bool["false"], __k);
          } else {
            return __lumo_match_error(__cps_v_313);
          }
        });
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
      return emit_token_accessor(__caps, s, label, t, Bool["true"], __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        return list_contains_string(__caps, token_defs, name, (__cps_v_314) => {
          if ((__cps_v_314[LUMO_TAG] === "true")) {
            return emit_token_accessor(__caps, s, label, TokenRef["named"](name), Bool["true"], __k);
          } else if ((__cps_v_314[LUMO_TAG] === "false")) {
            return emit_node_accessor(__caps, s, label, name, Bool["true"], __k);
          } else {
            return __lumo_match_error(__cps_v_314);
          }
        });
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
      return keyword_variant(__caps, kw, __k);
    } else {
      return ((t[LUMO_TAG] === "symbol") ? ((sym) => {
        return symbol_variant(__caps, sym, __k);
      })(t.args[0]) : __lumo_match_error(t));
    }
  });
}

export function emit_token_accessor(__caps, s, label, t, repeated, __k) {
  return token_kind_from_ref(__caps, t, (kind) => {
    if ((repeated[LUMO_TAG] === "true")) {
      return __caps.Add_String.add(__caps, s, "    pub fn ", (__cps_v_322) => {
        return __caps.Add_String.add(__caps, __cps_v_322, label, (__cps_v_321) => {
          return __caps.Add_String.add(__caps, __cps_v_321, "(&self) -> impl Iterator<Item = &'a LosslessToken> + 'a {\n", (s) => {
            return __caps.Add_String.add(__caps, s, "        self.0.children.iter().filter_map(|c| match c {\n", (s) => {
              return __caps.Add_String.add(__caps, s, "            SyntaxElement::Token(t) if t.kind == SyntaxKind::", (__cps_v_320) => {
                return __caps.Add_String.add(__caps, __cps_v_320, kind, (__cps_v_319) => {
                  return __caps.Add_String.add(__caps, __cps_v_319, " => Some(t),\n", (s) => {
                    return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
                      return __caps.Add_String.add(__caps, s, "        })\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "    }\n", __k);
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    } else if ((repeated[LUMO_TAG] === "false")) {
      return __caps.Add_String.add(__caps, s, "    pub fn ", (__cps_v_318) => {
        return __caps.Add_String.add(__caps, __cps_v_318, label, (__cps_v_317) => {
          return __caps.Add_String.add(__caps, __cps_v_317, "(&self) -> Option<&'a LosslessToken> {\n", (s) => {
            return __caps.Add_String.add(__caps, s, "        self.0.children.iter().find_map(|c| match c {\n", (s) => {
              return __caps.Add_String.add(__caps, s, "            SyntaxElement::Token(t) if t.kind == SyntaxKind::", (__cps_v_316) => {
                return __caps.Add_String.add(__caps, __cps_v_316, kind, (__cps_v_315) => {
                  return __caps.Add_String.add(__caps, __cps_v_315, " => Some(t),\n", (s) => {
                    return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
                      return __caps.Add_String.add(__caps, s, "        })\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "    }\n", __k);
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(repeated);
    }
  });
}

export function emit_node_accessor(__caps, s, label, node_name, repeated, __k) {
  return __thunk(() => {
    if ((repeated[LUMO_TAG] === "true")) {
      return __caps.Add_String.add(__caps, s, "    pub fn ", (__cps_v_334) => {
        return __caps.Add_String.add(__caps, __cps_v_334, label, (__cps_v_333) => {
          return __caps.Add_String.add(__caps, __cps_v_333, "(&self) -> impl Iterator<Item = ", (__cps_v_332) => {
            return __caps.Add_String.add(__caps, __cps_v_332, node_name, (__cps_v_331) => {
              return __caps.Add_String.add(__caps, __cps_v_331, "<'a>> + 'a {\n", (s) => {
                return __caps.Add_String.add(__caps, s, "        self.0.children.iter().filter_map(|c| match c {\n", (s) => {
                  return __caps.Add_String.add(__caps, s, "            SyntaxElement::Node(n) => ", (__cps_v_330) => {
                    return __caps.Add_String.add(__caps, __cps_v_330, node_name, (__cps_v_329) => {
                      return __caps.Add_String.add(__caps, __cps_v_329, "::cast(n),\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
                          return __caps.Add_String.add(__caps, s, "        })\n", (s) => {
                            return __caps.Add_String.add(__caps, s, "    }\n", __k);
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    } else if ((repeated[LUMO_TAG] === "false")) {
      return __caps.Add_String.add(__caps, s, "    pub fn ", (__cps_v_328) => {
        return __caps.Add_String.add(__caps, __cps_v_328, label, (__cps_v_327) => {
          return __caps.Add_String.add(__caps, __cps_v_327, "(&self) -> Option<", (__cps_v_326) => {
            return __caps.Add_String.add(__caps, __cps_v_326, node_name, (__cps_v_325) => {
              return __caps.Add_String.add(__caps, __cps_v_325, "<'a>> {\n", (s) => {
                return __caps.Add_String.add(__caps, s, "        self.0.children.iter().find_map(|c| match c {\n", (s) => {
                  return __caps.Add_String.add(__caps, s, "            SyntaxElement::Node(n) => ", (__cps_v_324) => {
                    return __caps.Add_String.add(__caps, __cps_v_324, node_name, (__cps_v_323) => {
                      return __caps.Add_String.add(__caps, __cps_v_323, "::cast(n),\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "            _ => None,\n", (s) => {
                          return __caps.Add_String.add(__caps, s, "        })\n", (s) => {
                            return __caps.Add_String.add(__caps, s, "    }\n", __k);
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    } else {
      return __lumo_match_error(repeated);
    }
  });
}

export function emit_enum_node(__caps, s, name, alts, __k) {
  return __thunk(() => {
    return __caps.Add_String.add(__caps, s, "pub enum ", (__cps_v_338) => {
      return __caps.Add_String.add(__caps, __cps_v_338, name, (__cps_v_337) => {
        return __caps.Add_String.add(__caps, __cps_v_337, "<'a> {\n", (s) => {
          return emit_enum_variants(__caps, s, alts, (s) => {
            return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
              return __caps.Add_String.add(__caps, s, "impl<'a> AstNode<'a> for ", (__cps_v_336) => {
                return __caps.Add_String.add(__caps, __cps_v_336, name, (__cps_v_335) => {
                  return __caps.Add_String.add(__caps, __cps_v_335, "<'a> {\n", (s) => {
                    return __caps.Add_String.add(__caps, s, "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n", (s) => {
                      return __caps.Add_String.add(__caps, s, "        None\n", (s) => {
                        return emit_enum_cast_chain(__caps, s, alts, (s) => {
                          return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                            return __caps.Add_String.add(__caps, s, "    fn syntax(&self) -> &'a SyntaxNode {\n", (s) => {
                              return __caps.Add_String.add(__caps, s, "        match self {\n", (s) => {
                                return emit_enum_syntax_arms(__caps, s, alts, (s) => {
                                  return __caps.Add_String.add(__caps, s, "        }\n", (s) => {
                                    return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                                      return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
                                        return __k(s);
                                      });
                                    });
                                  });
                                });
                              });
                            });
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    });
  });
}

export function emit_enum_variants(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return __caps.Add_String.add(__caps, s, "    ", (__cps_v_343) => {
          return __caps.Add_String.add(__caps, __cps_v_343, name, (__cps_v_342) => {
            return __caps.Add_String.add(__caps, __cps_v_342, "(", (__cps_v_341) => {
              return __caps.Add_String.add(__caps, __cps_v_341, name, (__cps_v_340) => {
                return __caps.Add_String.add(__caps, __cps_v_340, "<'a>),\n", (__cps_v_339) => {
                  return emit_enum_variants(__caps, __cps_v_339, rest, __k);
                });
              });
            });
          });
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_enum_cast_chain(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return __caps.Add_String.add(__caps, "            .or_else(|| ", name, (__cps_v_347) => {
          return __caps.Add_String.add(__caps, __cps_v_347, "::cast(node).map(Self::", (__cps_v_346) => {
            return __caps.Add_String.add(__caps, __cps_v_346, name, (__cps_v_345) => {
              return __caps.Add_String.add(__caps, __cps_v_345, "))\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_344) => {
                  return emit_enum_cast_chain(__caps, __cps_v_344, rest, __k);
                });
              });
            });
          });
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_enum_syntax_arms(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return __caps.Add_String.add(__caps, "            Self::", name, (__cps_v_349) => {
          return __caps.Add_String.add(__caps, __cps_v_349, "(n) => n.syntax(),\n", (line) => {
            return __caps.Add_String.add(__caps, s, line, (__cps_v_348) => {
              return emit_enum_syntax_arms(__caps, __cps_v_348, rest, __k);
            });
          });
        });
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_token_wrapper_node(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    return __caps.Add_String.add(__caps, s, "pub struct ", (__cps_v_355) => {
      return __caps.Add_String.add(__caps, __cps_v_355, name, (__cps_v_354) => {
        return __caps.Add_String.add(__caps, __cps_v_354, "<'a>(pub(crate) &'a SyntaxNode);\n\n", (s) => {
          return __caps.Add_String.add(__caps, s, "impl<'a> AstNode<'a> for ", (__cps_v_353) => {
            return __caps.Add_String.add(__caps, __cps_v_353, name, (__cps_v_352) => {
              return __caps.Add_String.add(__caps, __cps_v_352, "<'a> {\n", (s) => {
                return __caps.Add_String.add(__caps, s, "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n", (s) => {
                  return __caps.Add_String.add(__caps, s, "        (node.kind == SyntaxKind::", (__cps_v_351) => {
                    return __caps.Add_String.add(__caps, __cps_v_351, kind, (__cps_v_350) => {
                      return __caps.Add_String.add(__caps, __cps_v_350, ").then(|| Self(node))\n", (s) => {
                        return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                          return __caps.Add_String.add(__caps, s, "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n", (s) => {
                            return __caps.Add_String.add(__caps, s, "}\n\n", (s) => {
                              return __k(s);
                            });
                          });
                        });
                      });
                    });
                  });
                });
              });
            });
          });
        });
      });
    });
  });
}

main();
