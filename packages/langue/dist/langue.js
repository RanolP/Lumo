const LUMO_TAG = Symbol.for("Lumo/tag");
const __lumo_match_error = (value) => { throw new Error("non-exhaustive match: " + JSON.stringify(value)); };
const __thunk = (fn) => { fn.__t = 1; return fn; };
const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };
const __identity = (__v) => __v;

import { readFileSync as __lumo_readFileSync, writeFileSync as __lumo_writeFileSync } from "node:fs";



export function to_screaming_snake(__caps, name, __k) {
  return to_screaming_snake_loop__lto(__caps, name, 0, "", __k);
}

export function to_screaming_snake_loop(__caps, name, i, acc, __k) {
  return String.len(__caps, name, (__cps_v_27) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_27, (__cps_v_26) => {
      const __k_15 = (__cps_v_25) => {
        if ((__cps_v_25[LUMO_TAG] === "true")) {
          return __k(acc);
        } else if ((__cps_v_25[LUMO_TAG] === "false")) {
          return String.char_at(__caps, name, i, (c) => {
            return String.char_code_at(__caps, c, 0, (code) => {
              return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_24) => {
                const __k_14 = (__cps_v_23) => {
                  const __k_12 = (is_upper) => {
                    if ((is_upper[LUMO_TAG] === "true")) {
                      return __caps.PartialOrd_Number.cmp(__caps, 0, i, (__cps_v_21) => {
                        const __k_11 = (__cps_v_20) => {
                          if ((__cps_v_20[LUMO_TAG] === "true")) {
                            return __caps.Sub_Number.sub(__caps, i, 1, (__cps_v_19) => {
                              return String.char_at(__caps, name, __cps_v_19, (__cps_v_18) => {
                                return String.char_code_at(__caps, __cps_v_18, 0, (prev_code) => {
                                  return __caps.PartialOrd_Number.cmp(__caps, prev_code, 97, (__cps_v_17) => {
                                    const __k_10 = (__cps_v_16) => {
                                      const __k_8 = (prev_lower) => {
                                        return __caps.PartialOrd_Number.cmp(__caps, prev_code, 48, (__cps_v_14) => {
                                          const __k_7 = (__cps_v_13) => {
                                            const __k_5 = (prev_digit) => {
                                              if ((prev_lower[LUMO_TAG] === "true")) {
                                                return __caps.Add_Number.add(__caps, i, 1, (__cps_v_9) => {
                                                  return __caps.Add_String.add(__caps, acc, "_", (__cps_v_11) => {
                                                    return __caps.Add_String.add(__caps, __cps_v_11, to_upper_char__lto(c), (__cps_v_10) => {
                                                      return to_screaming_snake_loop__lto(__caps, name, __cps_v_9, __cps_v_10, __k);
                                                    });
                                                  });
                                                });
                                              } else if ((prev_lower[LUMO_TAG] === "false")) {
                                                if ((prev_digit[LUMO_TAG] === "true")) {
                                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_6) => {
                                                    return __caps.Add_String.add(__caps, acc, "_", (__cps_v_8) => {
                                                      return __caps.Add_String.add(__caps, __cps_v_8, to_upper_char__lto(c), (__cps_v_7) => {
                                                        return to_screaming_snake_loop__lto(__caps, name, __cps_v_6, __cps_v_7, __k);
                                                      });
                                                    });
                                                  });
                                                } else if ((prev_digit[LUMO_TAG] === "false")) {
                                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_4) => {
                                                    return __caps.Add_String.add(__caps, acc, to_upper_char__lto(c), (__cps_v_5) => {
                                                      return to_screaming_snake_loop__lto(__caps, name, __cps_v_4, __cps_v_5, __k);
                                                    });
                                                  });
                                                } else {
                                                  return __lumo_match_error(prev_digit);
                                                }
                                              } else {
                                                return __lumo_match_error(prev_lower);
                                              }
                                            };
                                            if ((__cps_v_13[LUMO_TAG] === "true")) {
                                              return __caps.PartialOrd_Number.cmp(__caps, prev_code, 57, (__cps_v_12) => {
                                                if ((__cps_v_12[LUMO_TAG] === "less")) {
                                                  return __k_5(Bool["true"]);
                                                } else if ((__cps_v_12[LUMO_TAG] === "equal")) {
                                                  return __k_5(Bool["true"]);
                                                } else {
                                                  return ((__cps_v_12[LUMO_TAG] === "greater") ? __k_5(Bool["false"]) : __lumo_match_error(__cps_v_12));
                                                }
                                              });
                                            } else if ((__cps_v_13[LUMO_TAG] === "false")) {
                                              return __k_5(Bool["false"]);
                                            } else {
                                              return __lumo_match_error(__cps_v_13);
                                            }
                                          };
                                          if ((__cps_v_14[LUMO_TAG] === "less")) {
                                            return __k_7(Bool["false"]);
                                          } else if ((__cps_v_14[LUMO_TAG] === "equal")) {
                                            return __k_7(Bool["true"]);
                                          } else {
                                            return ((__cps_v_14[LUMO_TAG] === "greater") ? __k_7(Bool["true"]) : __lumo_match_error(__cps_v_14));
                                          }
                                        });
                                      };
                                      if ((__cps_v_16[LUMO_TAG] === "true")) {
                                        return __caps.PartialOrd_Number.cmp(__caps, prev_code, 122, (__cps_v_15) => {
                                          if ((__cps_v_15[LUMO_TAG] === "less")) {
                                            return __k_8(Bool["true"]);
                                          } else if ((__cps_v_15[LUMO_TAG] === "equal")) {
                                            return __k_8(Bool["true"]);
                                          } else {
                                            return ((__cps_v_15[LUMO_TAG] === "greater") ? __k_8(Bool["false"]) : __lumo_match_error(__cps_v_15));
                                          }
                                        });
                                      } else if ((__cps_v_16[LUMO_TAG] === "false")) {
                                        return __k_8(Bool["false"]);
                                      } else {
                                        return __lumo_match_error(__cps_v_16);
                                      }
                                    };
                                    if ((__cps_v_17[LUMO_TAG] === "less")) {
                                      return __k_10(Bool["false"]);
                                    } else if ((__cps_v_17[LUMO_TAG] === "equal")) {
                                      return __k_10(Bool["true"]);
                                    } else {
                                      return ((__cps_v_17[LUMO_TAG] === "greater") ? __k_10(Bool["true"]) : __lumo_match_error(__cps_v_17));
                                    }
                                  });
                                });
                              });
                            });
                          } else if ((__cps_v_20[LUMO_TAG] === "false")) {
                            return __caps.Add_Number.add(__caps, i, 1, (__cps_v_2) => {
                              return __caps.Add_String.add(__caps, acc, to_upper_char__lto(c), (__cps_v_3) => {
                                return to_screaming_snake_loop__lto(__caps, name, __cps_v_2, __cps_v_3, __k);
                              });
                            });
                          } else {
                            return __lumo_match_error(__cps_v_20);
                          }
                        };
                        if ((__cps_v_21[LUMO_TAG] === "less")) {
                          return __k_11(Bool["true"]);
                        } else if ((__cps_v_21[LUMO_TAG] === "equal")) {
                          return __k_11(Bool["false"]);
                        } else {
                          return ((__cps_v_21[LUMO_TAG] === "greater") ? __k_11(Bool["false"]) : __lumo_match_error(__cps_v_21));
                        }
                      });
                    } else if ((is_upper[LUMO_TAG] === "false")) {
                      return __caps.Add_Number.add(__caps, i, 1, (__cps_v_0) => {
                        return __caps.Add_String.add(__caps, acc, to_upper_char__lto(c), (__cps_v_1) => {
                          return to_screaming_snake_loop__lto(__caps, name, __cps_v_0, __cps_v_1, __k);
                        });
                      });
                    } else {
                      return __lumo_match_error(is_upper);
                    }
                  };
                  if ((__cps_v_23[LUMO_TAG] === "true")) {
                    return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_22) => {
                      if ((__cps_v_22[LUMO_TAG] === "less")) {
                        return __k_12(Bool["true"]);
                      } else if ((__cps_v_22[LUMO_TAG] === "equal")) {
                        return __k_12(Bool["true"]);
                      } else {
                        return ((__cps_v_22[LUMO_TAG] === "greater") ? __k_12(Bool["false"]) : __lumo_match_error(__cps_v_22));
                      }
                    });
                  } else if ((__cps_v_23[LUMO_TAG] === "false")) {
                    return __k_12(Bool["false"]);
                  } else {
                    return __lumo_match_error(__cps_v_23);
                  }
                };
                if ((__cps_v_24[LUMO_TAG] === "less")) {
                  return __k_14(Bool["false"]);
                } else if ((__cps_v_24[LUMO_TAG] === "equal")) {
                  return __k_14(Bool["true"]);
                } else {
                  return ((__cps_v_24[LUMO_TAG] === "greater") ? __k_14(Bool["true"]) : __lumo_match_error(__cps_v_24));
                }
              });
            });
          });
        } else {
          return __lumo_match_error(__cps_v_25);
        }
      };
      if ((__cps_v_26[LUMO_TAG] === "less")) {
        return __k_15(Bool["false"]);
      } else if ((__cps_v_26[LUMO_TAG] === "equal")) {
        return __k_15(Bool["true"]);
      } else {
        return ((__cps_v_26[LUMO_TAG] === "greater") ? __k_15(Bool["true"]) : __lumo_match_error(__cps_v_26));
      }
    });
  });
}

export function to_upper_char(__caps, c, __k) {
  return String.char_code_at(__caps, c, 0, (code) => {
    return __caps.PartialOrd_Number.cmp(__caps, code, 97, (__cps_v_32) => {
      const __k_19 = (__cps_v_31) => {
        if ((__cps_v_31[LUMO_TAG] === "true")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 122, (__cps_v_30) => {
            const __k_18 = (__cps_v_29) => {
              if ((__cps_v_29[LUMO_TAG] === "true")) {
                return __caps.Sub_Number.sub(__caps, code, 32, (__cps_v_28) => {
                  return __caps.StrOps_StrOps.from_char_code(__caps, __cps_v_28, __k);
                });
              } else if ((__cps_v_29[LUMO_TAG] === "false")) {
                return __k(c);
              } else {
                return __lumo_match_error(__cps_v_29);
              }
            };
            if ((__cps_v_30[LUMO_TAG] === "less")) {
              return __k_18(Bool["true"]);
            } else if ((__cps_v_30[LUMO_TAG] === "equal")) {
              return __k_18(Bool["true"]);
            } else {
              return ((__cps_v_30[LUMO_TAG] === "greater") ? __k_18(Bool["false"]) : __lumo_match_error(__cps_v_30));
            }
          });
        } else if ((__cps_v_31[LUMO_TAG] === "false")) {
          return __k(c);
        } else {
          return __lumo_match_error(__cps_v_31);
        }
      };
      if ((__cps_v_32[LUMO_TAG] === "less")) {
        return __k_19(Bool["false"]);
      } else if ((__cps_v_32[LUMO_TAG] === "equal")) {
        return __k_19(Bool["true"]);
      } else {
        return ((__cps_v_32[LUMO_TAG] === "greater") ? __k_19(Bool["true"]) : __lumo_match_error(__cps_v_32));
      }
    });
  });
}

export function keyword_variant(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__cps_v_33) => {
    return __caps.Add_String.add(__caps, __cps_v_33, "_KW", __k);
  });
}

export function to_upper_string(__caps, s, __k) {
  return to_upper_string_loop__lto(__caps, s, 0, "", __k);
}

export function to_upper_string_loop(__caps, s, i, acc, __k) {
  return String.len(__caps, s, (__cps_v_40) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_40, (__cps_v_39) => {
      const __k_21 = (__cps_v_38) => {
        if ((__cps_v_38[LUMO_TAG] === "true")) {
          return __k(acc);
        } else if ((__cps_v_38[LUMO_TAG] === "false")) {
          return __caps.Add_Number.add(__caps, i, 1, (__cps_v_34) => {
            return String.char_at(__caps, s, i, (__cps_v_37) => {
              const __cps_v_36 = to_upper_char__lto(__cps_v_37);
              return __caps.Add_String.add(__caps, acc, __cps_v_36, (__cps_v_35) => {
                return to_upper_string_loop__lto(__caps, s, __cps_v_34, __cps_v_35, __k);
              });
            });
          });
        } else {
          return __lumo_match_error(__cps_v_38);
        }
      };
      if ((__cps_v_39[LUMO_TAG] === "less")) {
        return __k_21(Bool["false"]);
      } else if ((__cps_v_39[LUMO_TAG] === "equal")) {
        return __k_21(Bool["true"]);
      } else {
        return ((__cps_v_39[LUMO_TAG] === "greater") ? __k_21(Bool["true"]) : __lumo_match_error(__cps_v_39));
      }
    });
  });
}

export function symbol_variant(__caps, sym, __k) {
  return __thunk(() => {
    return __caps.PartialEq_String.eq(__caps, sym, "#", (__cps_v_70) => {
      if ((__cps_v_70[LUMO_TAG] === "true")) {
        return __k("HASH");
      } else if ((__cps_v_70[LUMO_TAG] === "false")) {
        return __caps.PartialEq_String.eq(__caps, sym, "(", (__cps_v_69) => {
          if ((__cps_v_69[LUMO_TAG] === "true")) {
            return __k("L_PAREN");
          } else if ((__cps_v_69[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, sym, ")", (__cps_v_68) => {
              if ((__cps_v_68[LUMO_TAG] === "true")) {
                return __k("R_PAREN");
              } else if ((__cps_v_68[LUMO_TAG] === "false")) {
                return __caps.PartialEq_String.eq(__caps, sym, "[", (__cps_v_67) => {
                  if ((__cps_v_67[LUMO_TAG] === "true")) {
                    return __k("L_BRACKET");
                  } else if ((__cps_v_67[LUMO_TAG] === "false")) {
                    return __caps.PartialEq_String.eq(__caps, sym, "]", (__cps_v_66) => {
                      if ((__cps_v_66[LUMO_TAG] === "true")) {
                        return __k("R_BRACKET");
                      } else if ((__cps_v_66[LUMO_TAG] === "false")) {
                        return __caps.PartialEq_String.eq(__caps, sym, "{", (__cps_v_65) => {
                          if ((__cps_v_65[LUMO_TAG] === "true")) {
                            return __k("L_BRACE");
                          } else if ((__cps_v_65[LUMO_TAG] === "false")) {
                            return __caps.PartialEq_String.eq(__caps, sym, "}", (__cps_v_64) => {
                              if ((__cps_v_64[LUMO_TAG] === "true")) {
                                return __k("R_BRACE");
                              } else if ((__cps_v_64[LUMO_TAG] === "false")) {
                                return __caps.PartialEq_String.eq(__caps, sym, ";", (__cps_v_63) => {
                                  if ((__cps_v_63[LUMO_TAG] === "true")) {
                                    return __k("SEMICOLON");
                                  } else if ((__cps_v_63[LUMO_TAG] === "false")) {
                                    return __caps.PartialEq_String.eq(__caps, sym, ":", (__cps_v_62) => {
                                      if ((__cps_v_62[LUMO_TAG] === "true")) {
                                        return __k("COLON");
                                      } else if ((__cps_v_62[LUMO_TAG] === "false")) {
                                        return __caps.PartialEq_String.eq(__caps, sym, ",", (__cps_v_61) => {
                                          if ((__cps_v_61[LUMO_TAG] === "true")) {
                                            return __k("COMMA");
                                          } else if ((__cps_v_61[LUMO_TAG] === "false")) {
                                            return __caps.PartialEq_String.eq(__caps, sym, "=", (__cps_v_60) => {
                                              if ((__cps_v_60[LUMO_TAG] === "true")) {
                                                return __k("EQUALS");
                                              } else if ((__cps_v_60[LUMO_TAG] === "false")) {
                                                return __caps.PartialEq_String.eq(__caps, sym, ":=", (__cps_v_59) => {
                                                  if ((__cps_v_59[LUMO_TAG] === "true")) {
                                                    return __k("COLON_EQ");
                                                  } else if ((__cps_v_59[LUMO_TAG] === "false")) {
                                                    return __caps.PartialEq_String.eq(__caps, sym, "=>", (__cps_v_58) => {
                                                      if ((__cps_v_58[LUMO_TAG] === "true")) {
                                                        return __k("FAT_ARROW");
                                                      } else if ((__cps_v_58[LUMO_TAG] === "false")) {
                                                        return __caps.PartialEq_String.eq(__caps, sym, "->", (__cps_v_57) => {
                                                          if ((__cps_v_57[LUMO_TAG] === "true")) {
                                                            return __k("ARROW");
                                                          } else if ((__cps_v_57[LUMO_TAG] === "false")) {
                                                            return __caps.PartialEq_String.eq(__caps, sym, ".", (__cps_v_56) => {
                                                              if ((__cps_v_56[LUMO_TAG] === "true")) {
                                                                return __k("DOT");
                                                              } else if ((__cps_v_56[LUMO_TAG] === "false")) {
                                                                return __caps.PartialEq_String.eq(__caps, sym, "+", (__cps_v_55) => {
                                                                  if ((__cps_v_55[LUMO_TAG] === "true")) {
                                                                    return __k("PLUS");
                                                                  } else if ((__cps_v_55[LUMO_TAG] === "false")) {
                                                                    return __caps.PartialEq_String.eq(__caps, sym, "-", (__cps_v_54) => {
                                                                      if ((__cps_v_54[LUMO_TAG] === "true")) {
                                                                        return __k("MINUS");
                                                                      } else if ((__cps_v_54[LUMO_TAG] === "false")) {
                                                                        return __caps.PartialEq_String.eq(__caps, sym, "*", (__cps_v_53) => {
                                                                          if ((__cps_v_53[LUMO_TAG] === "true")) {
                                                                            return __k("STAR");
                                                                          } else if ((__cps_v_53[LUMO_TAG] === "false")) {
                                                                            return __caps.PartialEq_String.eq(__caps, sym, "/", (__cps_v_52) => {
                                                                              if ((__cps_v_52[LUMO_TAG] === "true")) {
                                                                                return __k("SLASH");
                                                                              } else if ((__cps_v_52[LUMO_TAG] === "false")) {
                                                                                return __caps.PartialEq_String.eq(__caps, sym, "%", (__cps_v_51) => {
                                                                                  if ((__cps_v_51[LUMO_TAG] === "true")) {
                                                                                    return __k("PERCENT");
                                                                                  } else if ((__cps_v_51[LUMO_TAG] === "false")) {
                                                                                    return __caps.PartialEq_String.eq(__caps, sym, "!", (__cps_v_50) => {
                                                                                      if ((__cps_v_50[LUMO_TAG] === "true")) {
                                                                                        return __k("BANG");
                                                                                      } else if ((__cps_v_50[LUMO_TAG] === "false")) {
                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "<", (__cps_v_49) => {
                                                                                          if ((__cps_v_49[LUMO_TAG] === "true")) {
                                                                                            return __k("LT");
                                                                                          } else if ((__cps_v_49[LUMO_TAG] === "false")) {
                                                                                            return __caps.PartialEq_String.eq(__caps, sym, ">", (__cps_v_48) => {
                                                                                              if ((__cps_v_48[LUMO_TAG] === "true")) {
                                                                                                return __k("GT");
                                                                                              } else if ((__cps_v_48[LUMO_TAG] === "false")) {
                                                                                                return __caps.PartialEq_String.eq(__caps, sym, "<=", (__cps_v_47) => {
                                                                                                  if ((__cps_v_47[LUMO_TAG] === "true")) {
                                                                                                    return __k("LT_EQ");
                                                                                                  } else if ((__cps_v_47[LUMO_TAG] === "false")) {
                                                                                                    return __caps.PartialEq_String.eq(__caps, sym, ">=", (__cps_v_46) => {
                                                                                                      if ((__cps_v_46[LUMO_TAG] === "true")) {
                                                                                                        return __k("GT_EQ");
                                                                                                      } else if ((__cps_v_46[LUMO_TAG] === "false")) {
                                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "==", (__cps_v_45) => {
                                                                                                          if ((__cps_v_45[LUMO_TAG] === "true")) {
                                                                                                            return __k("EQ_EQ");
                                                                                                          } else if ((__cps_v_45[LUMO_TAG] === "false")) {
                                                                                                            return __caps.PartialEq_String.eq(__caps, sym, "!=", (__cps_v_44) => {
                                                                                                              if ((__cps_v_44[LUMO_TAG] === "true")) {
                                                                                                                return __k("BANG_EQ");
                                                                                                              } else if ((__cps_v_44[LUMO_TAG] === "false")) {
                                                                                                                return __caps.PartialEq_String.eq(__caps, sym, "&&", (__cps_v_43) => {
                                                                                                                  if ((__cps_v_43[LUMO_TAG] === "true")) {
                                                                                                                    return __k("AMP_AMP");
                                                                                                                  } else if ((__cps_v_43[LUMO_TAG] === "false")) {
                                                                                                                    return __caps.PartialEq_String.eq(__caps, sym, "||", (__cps_v_42) => {
                                                                                                                      if ((__cps_v_42[LUMO_TAG] === "true")) {
                                                                                                                        return __k("PIPE_PIPE");
                                                                                                                      } else if ((__cps_v_42[LUMO_TAG] === "false")) {
                                                                                                                        return __caps.PartialEq_String.eq(__caps, sym, "_", (__cps_v_41) => {
                                                                                                                          if ((__cps_v_41[LUMO_TAG] === "true")) {
                                                                                                                            return __k("UNDERSCORE");
                                                                                                                          } else if ((__cps_v_41[LUMO_TAG] === "false")) {
                                                                                                                            return __caps.Add_String.add(__caps, "SYM_", sym, __k);
                                                                                                                          } else {
                                                                                                                            return __lumo_match_error(__cps_v_41);
                                                                                                                          }
                                                                                                                        });
                                                                                                                      } else {
                                                                                                                        return __lumo_match_error(__cps_v_42);
                                                                                                                      }
                                                                                                                    });
                                                                                                                  } else {
                                                                                                                    return __lumo_match_error(__cps_v_43);
                                                                                                                  }
                                                                                                                });
                                                                                                              } else {
                                                                                                                return __lumo_match_error(__cps_v_44);
                                                                                                              }
                                                                                                            });
                                                                                                          } else {
                                                                                                            return __lumo_match_error(__cps_v_45);
                                                                                                          }
                                                                                                        });
                                                                                                      } else {
                                                                                                        return __lumo_match_error(__cps_v_46);
                                                                                                      }
                                                                                                    });
                                                                                                  } else {
                                                                                                    return __lumo_match_error(__cps_v_47);
                                                                                                  }
                                                                                                });
                                                                                              } else {
                                                                                                return __lumo_match_error(__cps_v_48);
                                                                                              }
                                                                                            });
                                                                                          } else {
                                                                                            return __lumo_match_error(__cps_v_49);
                                                                                          }
                                                                                        });
                                                                                      } else {
                                                                                        return __lumo_match_error(__cps_v_50);
                                                                                      }
                                                                                    });
                                                                                  } else {
                                                                                    return __lumo_match_error(__cps_v_51);
                                                                                  }
                                                                                });
                                                                              } else {
                                                                                return __lumo_match_error(__cps_v_52);
                                                                              }
                                                                            });
                                                                          } else {
                                                                            return __lumo_match_error(__cps_v_53);
                                                                          }
                                                                        });
                                                                      } else {
                                                                        return __lumo_match_error(__cps_v_54);
                                                                      }
                                                                    });
                                                                  } else {
                                                                    return __lumo_match_error(__cps_v_55);
                                                                  }
                                                                });
                                                              } else {
                                                                return __lumo_match_error(__cps_v_56);
                                                              }
                                                            });
                                                          } else {
                                                            return __lumo_match_error(__cps_v_57);
                                                          }
                                                        });
                                                      } else {
                                                        return __lumo_match_error(__cps_v_58);
                                                      }
                                                    });
                                                  } else {
                                                    return __lumo_match_error(__cps_v_59);
                                                  }
                                                });
                                              } else {
                                                return __lumo_match_error(__cps_v_60);
                                              }
                                            });
                                          } else {
                                            return __lumo_match_error(__cps_v_61);
                                          }
                                        });
                                      } else {
                                        return __lumo_match_error(__cps_v_62);
                                      }
                                    });
                                  } else {
                                    return __lumo_match_error(__cps_v_63);
                                  }
                                });
                              } else {
                                return __lumo_match_error(__cps_v_64);
                              }
                            });
                          } else {
                            return __lumo_match_error(__cps_v_65);
                          }
                        });
                      } else {
                        return __lumo_match_error(__cps_v_66);
                      }
                    });
                  } else {
                    return __lumo_match_error(__cps_v_67);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_68);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_69);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_70);
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
          return dedupe_strings(__caps, kws, (__cps_v_74) => {
            return sort_strings(__caps, __cps_v_74, (__cps_v_71) => {
              return dedupe_strings(__caps, syms, (__cps_v_73) => {
                return sort_strings(__caps, __cps_v_73, (__cps_v_72) => {
                  return __k(CollectedTokens["mk"](__cps_v_71, __cps_v_72));
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
      return collect_tokens_from_alts__lto(__caps, alts, kws, syms, __k);
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
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_78) => {
            const __k_63 = (__cps_v_77) => {
              if ((__cps_v_77[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_76) => {
                  const __k_62 = (__cps_v_75) => {
                    if ((__cps_v_75[LUMO_TAG] === "true")) {
                      return collect_tokens_from_alts__lto(__caps, rest, kws, syms, __k);
                    } else if ((__cps_v_75[LUMO_TAG] === "false")) {
                      return collect_alt_token(__caps, name, rest, kws, syms, __k);
                    } else {
                      return __lumo_match_error(__cps_v_75);
                    }
                  };
                  if ((__cps_v_76[LUMO_TAG] === "less")) {
                    return __k_62(Bool["true"]);
                  } else if ((__cps_v_76[LUMO_TAG] === "equal")) {
                    return __k_62(Bool["true"]);
                  } else {
                    return ((__cps_v_76[LUMO_TAG] === "greater") ? __k_62(Bool["false"]) : __lumo_match_error(__cps_v_76));
                  }
                });
              } else if ((__cps_v_77[LUMO_TAG] === "false")) {
                return collect_alt_token(__caps, name, rest, kws, syms, __k);
              } else {
                return __lumo_match_error(__cps_v_77);
              }
            };
            if ((__cps_v_78[LUMO_TAG] === "less")) {
              return __k_63(Bool["false"]);
            } else if ((__cps_v_78[LUMO_TAG] === "equal")) {
              return __k_63(Bool["true"]);
            } else {
              return ((__cps_v_78[LUMO_TAG] === "greater") ? __k_63(Bool["true"]) : __lumo_match_error(__cps_v_78));
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
  return has_alpha__lto(__caps, name, 0, (__cps_v_79) => {
    if ((__cps_v_79[LUMO_TAG] === "true")) {
      return collect_tokens_from_alts__lto(__caps, rest, List["cons"](name, kws), syms, __k);
    } else if ((__cps_v_79[LUMO_TAG] === "false")) {
      return collect_tokens_from_alts__lto(__caps, rest, kws, List["cons"](name, syms), __k);
    } else {
      return __lumo_match_error(__cps_v_79);
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
      return list_contains_string__lto(__caps, acc, x, (__cps_v_80) => {
        if ((__cps_v_80[LUMO_TAG] === "true")) {
          return dedupe_strings_acc(__caps, rest, acc, __k);
        } else if ((__cps_v_80[LUMO_TAG] === "false")) {
          return dedupe_strings_acc(__caps, rest, List["cons"](x, acc), __k);
        } else {
          return __lumo_match_error(__cps_v_80);
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
      return insert_sorted(__caps, x, sorted, (__cps_v_81) => {
        return sort_strings_acc(__caps, rest, __cps_v_81, __k);
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
      return string_lt(__caps, s, x, (__cps_v_83) => {
        if ((__cps_v_83[LUMO_TAG] === "true")) {
          return __k(List["cons"](s, xs));
        } else if ((__cps_v_83[LUMO_TAG] === "false")) {
          return insert_sorted(__caps, s, rest, (__cps_v_82) => {
            return __k(List["cons"](x, __cps_v_82));
          });
        } else {
          return __lumo_match_error(__cps_v_83);
        }
      });
    } else {
      return __lumo_match_error(xs);
    }
  });
}

export function string_lt(__caps, a, b, __k) {
  return string_lt_loop__lto(__caps, a, b, 0, __k);
}

export function string_lt_loop(__caps, a, b, i, __k) {
  return String.len(__caps, a, (__cps_v_97) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_97, (__cps_v_96) => {
      const __k_79 = (__cps_v_95) => {
        if ((__cps_v_95[LUMO_TAG] === "true")) {
          return String.len(__caps, b, (__cps_v_94) => {
            return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_94, (__cps_v_93) => {
              const __k_78 = (__cps_v_92) => {
                if ((__cps_v_92[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((__cps_v_92[LUMO_TAG] === "false")) {
                  return __k(Bool["true"]);
                } else {
                  return __lumo_match_error(__cps_v_92);
                }
              };
              if ((__cps_v_93[LUMO_TAG] === "less")) {
                return __k_78(Bool["false"]);
              } else if ((__cps_v_93[LUMO_TAG] === "equal")) {
                return __k_78(Bool["true"]);
              } else {
                return ((__cps_v_93[LUMO_TAG] === "greater") ? __k_78(Bool["true"]) : __lumo_match_error(__cps_v_93));
              }
            });
          });
        } else if ((__cps_v_95[LUMO_TAG] === "false")) {
          return String.len(__caps, b, (__cps_v_91) => {
            return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_91, (__cps_v_90) => {
              const __k_76 = (__cps_v_89) => {
                if ((__cps_v_89[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((__cps_v_89[LUMO_TAG] === "false")) {
                  return String.char_code_at(__caps, a, i, (ca) => {
                    return String.char_code_at(__caps, b, i, (cb) => {
                      return __caps.PartialOrd_Number.cmp(__caps, ca, cb, (__cps_v_88) => {
                        const __k_75 = (__cps_v_87) => {
                          if ((__cps_v_87[LUMO_TAG] === "true")) {
                            return __k(Bool["true"]);
                          } else if ((__cps_v_87[LUMO_TAG] === "false")) {
                            return __caps.PartialOrd_Number.cmp(__caps, cb, ca, (__cps_v_86) => {
                              const __k_74 = (__cps_v_85) => {
                                if ((__cps_v_85[LUMO_TAG] === "true")) {
                                  return __k(Bool["false"]);
                                } else if ((__cps_v_85[LUMO_TAG] === "false")) {
                                  return __caps.Add_Number.add(__caps, i, 1, (__cps_v_84) => {
                                    return string_lt_loop__lto(__caps, a, b, __cps_v_84, __k);
                                  });
                                } else {
                                  return __lumo_match_error(__cps_v_85);
                                }
                              };
                              if ((__cps_v_86[LUMO_TAG] === "less")) {
                                return __k_74(Bool["true"]);
                              } else if ((__cps_v_86[LUMO_TAG] === "equal")) {
                                return __k_74(Bool["false"]);
                              } else {
                                return ((__cps_v_86[LUMO_TAG] === "greater") ? __k_74(Bool["false"]) : __lumo_match_error(__cps_v_86));
                              }
                            });
                          } else {
                            return __lumo_match_error(__cps_v_87);
                          }
                        };
                        if ((__cps_v_88[LUMO_TAG] === "less")) {
                          return __k_75(Bool["true"]);
                        } else if ((__cps_v_88[LUMO_TAG] === "equal")) {
                          return __k_75(Bool["false"]);
                        } else {
                          return ((__cps_v_88[LUMO_TAG] === "greater") ? __k_75(Bool["false"]) : __lumo_match_error(__cps_v_88));
                        }
                      });
                    });
                  });
                } else {
                  return __lumo_match_error(__cps_v_89);
                }
              };
              if ((__cps_v_90[LUMO_TAG] === "less")) {
                return __k_76(Bool["false"]);
              } else if ((__cps_v_90[LUMO_TAG] === "equal")) {
                return __k_76(Bool["true"]);
              } else {
                return ((__cps_v_90[LUMO_TAG] === "greater") ? __k_76(Bool["true"]) : __lumo_match_error(__cps_v_90));
              }
            });
          });
        } else {
          return __lumo_match_error(__cps_v_95);
        }
      };
      if ((__cps_v_96[LUMO_TAG] === "less")) {
        return __k_79(Bool["false"]);
      } else if ((__cps_v_96[LUMO_TAG] === "equal")) {
        return __k_79(Bool["true"]);
      } else {
        return ((__cps_v_96[LUMO_TAG] === "greater") ? __k_79(Bool["true"]) : __lumo_match_error(__cps_v_96));
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
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_100) => {
            const __k_85 = (__cps_v_99) => {
              const __k_83 = (is_upper) => {
                if ((is_upper[LUMO_TAG] === "true")) {
                  return __k(Bool["false"]);
                } else if ((is_upper[LUMO_TAG] === "false")) {
                  return is_token_only_alternatives__lto(__caps, rest, __k);
                } else {
                  return __lumo_match_error(is_upper);
                }
              };
              if ((__cps_v_99[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_98) => {
                  if ((__cps_v_98[LUMO_TAG] === "less")) {
                    return __k_83(Bool["true"]);
                  } else if ((__cps_v_98[LUMO_TAG] === "equal")) {
                    return __k_83(Bool["true"]);
                  } else {
                    return ((__cps_v_98[LUMO_TAG] === "greater") ? __k_83(Bool["false"]) : __lumo_match_error(__cps_v_98));
                  }
                });
              } else if ((__cps_v_99[LUMO_TAG] === "false")) {
                return __k_83(Bool["false"]);
              } else {
                return __lumo_match_error(__cps_v_99);
              }
            };
            if ((__cps_v_100[LUMO_TAG] === "less")) {
              return __k_85(Bool["false"]);
            } else if ((__cps_v_100[LUMO_TAG] === "equal")) {
              return __k_85(Bool["true"]);
            } else {
              return ((__cps_v_100[LUMO_TAG] === "greater") ? __k_85(Bool["true"]) : __lumo_match_error(__cps_v_100));
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
                  return emit_named_tokens__lto(__caps, s, token_defs, (s) => {
                    return __caps.Add_String.add(__caps, s, "    // Trivia\n", (s) => {
                      return __caps.Add_String.add(__caps, s, "    WHITESPACE,\n    NEWLINE,\n    UNKNOWN,\n", (s) => {
                        return emit_keywords__lto(__caps, s, keywords, (s) => {
                          return emit_symbols__lto(__caps, s, symbols, (s) => {
                            return __caps.Add_String.add(__caps, s, "    // Nodes\n", (s) => {
                              return emit_node_kinds__lto(__caps, s, rules, (s) => {
                                return __caps.Add_String.add(__caps, s, "    // Sentinel\n    ERROR,\n", (s) => {
                                  return __caps.Add_String.add(__caps, s, "}\n", (s) => {
                                    return __caps.Add_String.add(__caps, s, "\nimpl SyntaxKind {\n", (s) => {
                                      return __caps.Add_String.add(__caps, s, "    pub fn is_trivia(self) -> bool {\n", (s) => {
                                        return __caps.Add_String.add(__caps, s, "        matches!(self, Self::WHITESPACE | Self::NEWLINE)\n", (s) => {
                                          return __caps.Add_String.add(__caps, s, "    }\n", (s) => {
                                            return emit_from_keyword__lto(__caps, s, keywords, (s) => {
                                              return emit_from_symbol__lto(__caps, s, symbols, (s) => {
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
      return __caps.Add_String.add(__caps, s, "    ", (__cps_v_103) => {
        return to_screaming_snake(__caps, tok, (__cps_v_104) => {
          return __caps.Add_String.add(__caps, __cps_v_103, __cps_v_104, (__cps_v_102) => {
            return __caps.Add_String.add(__caps, __cps_v_102, ",\n", (__cps_v_101) => {
              return emit_named_tokens__lto(__caps, __cps_v_101, rest, __k);
            });
          });
        });
      });
    } else {
      return __lumo_match_error(tokens);
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
      return keyword_variant__lto(__caps, kw, (__cps_v_109) => {
        return __caps.Add_String.add(__caps, "    ", __cps_v_109, (__cps_v_108) => {
          return __caps.Add_String.add(__caps, __cps_v_108, ", // '", (__cps_v_107) => {
            return __caps.Add_String.add(__caps, __cps_v_107, kw, (__cps_v_106) => {
              return __caps.Add_String.add(__caps, __cps_v_106, "'\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_105) => {
                  return emit_keywords_items__lto(__caps, __cps_v_105, rest, __k);
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

export function emit_symbols_items(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      return __caps.Add_String.add(__caps, "    ", symbol_variant__lto(sym), (__cps_v_113) => {
        return __caps.Add_String.add(__caps, __cps_v_113, ", // '", (__cps_v_112) => {
          return __caps.Add_String.add(__caps, __cps_v_112, sym, (__cps_v_111) => {
            return __caps.Add_String.add(__caps, __cps_v_111, "'\n", (line) => {
              return __caps.Add_String.add(__caps, s, line, (__cps_v_110) => {
                return emit_symbols_items__lto(__caps, __cps_v_110, rest, __k);
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
          return to_screaming_snake(__caps, name, (__cps_v_124) => {
            return __caps.Add_String.add(__caps, "    ", __cps_v_124, (__cps_v_123) => {
              return __caps.Add_String.add(__caps, __cps_v_123, ", // ", (__cps_v_122) => {
                return __caps.Add_String.add(__caps, __cps_v_122, name, (__cps_v_121) => {
                  return __caps.Add_String.add(__caps, __cps_v_121, "\n", (line) => {
                    return __caps.Add_String.add(__caps, s, line, (__cps_v_120) => {
                      return emit_node_kinds__lto(__caps, __cps_v_120, rest, __k);
                    });
                  });
                });
              });
            });
          });
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          return is_token_only_alternatives__lto(__caps, alts, (__cps_v_119) => {
            if ((__cps_v_119[LUMO_TAG] === "true")) {
              return to_screaming_snake(__caps, name, (__cps_v_118) => {
                return __caps.Add_String.add(__caps, "    ", __cps_v_118, (__cps_v_117) => {
                  return __caps.Add_String.add(__caps, __cps_v_117, ", // ", (__cps_v_116) => {
                    return __caps.Add_String.add(__caps, __cps_v_116, name, (__cps_v_115) => {
                      return __caps.Add_String.add(__caps, __cps_v_115, " (token wrapper)\n", (line) => {
                        return __caps.Add_String.add(__caps, s, line, (__cps_v_114) => {
                          return emit_node_kinds__lto(__caps, __cps_v_114, rest, __k);
                        });
                      });
                    });
                  });
                });
              });
            } else if ((__cps_v_119[LUMO_TAG] === "false")) {
              return emit_node_kinds__lto(__caps, s, rest, __k);
            } else {
              return __lumo_match_error(__cps_v_119);
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

export function emit_keyword_arms(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      return __caps.Add_String.add(__caps, "            \"", kw, (__cps_v_129) => {
        return __caps.Add_String.add(__caps, __cps_v_129, "\" => Some(Self::", (__cps_v_127) => {
          return keyword_variant__lto(__caps, kw, (__cps_v_128) => {
            return __caps.Add_String.add(__caps, __cps_v_127, __cps_v_128, (__cps_v_126) => {
              return __caps.Add_String.add(__caps, __cps_v_126, "),\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_125) => {
                  return emit_keyword_arms__lto(__caps, __cps_v_125, rest, __k);
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

export function emit_symbol_arms(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      return __caps.Add_String.add(__caps, "            \"", sym, (__cps_v_133) => {
        return __caps.Add_String.add(__caps, __cps_v_133, "\" => Some(Self::", (__cps_v_132) => {
          return __caps.Add_String.add(__caps, __cps_v_132, symbol_variant__lto(sym), (__cps_v_131) => {
            return __caps.Add_String.add(__caps, __cps_v_131, "),\n", (line) => {
              return __caps.Add_String.add(__caps, s, line, (__cps_v_130) => {
                return emit_symbol_arms__lto(__caps, __cps_v_130, rest, __k);
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
        const __k_100 = (s2) => {
          return emit_ast_rules(__caps, s2, token_defs, rest, __k);
        };
        if ((body[LUMO_TAG] === "sequence")) {
          const elems = body.args[0];
          return emit_struct_node__lto(__caps, s, name, elems, token_defs, __k_100);
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          return is_token_only_alternatives__lto(__caps, alts, (__cps_v_134) => {
            if ((__cps_v_134[LUMO_TAG] === "true")) {
              return emit_token_wrapper_node__lto(__caps, s, name, __k_100);
            } else if ((__cps_v_134[LUMO_TAG] === "false")) {
              return emit_enum_node__lto(__caps, s, name, alts, __k_100);
            } else {
              return __lumo_match_error(__cps_v_134);
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

export function emit_accessors(__caps, s, struct_name, elems, token_defs, __k) {
  return __thunk(() => {
    const has_labeled = has_labeled_elements(elems);
    if ((has_labeled[LUMO_TAG] === "true")) {
      return __caps.Add_String.add(__caps, s, "impl<'a> ", (__cps_v_136) => {
        return __caps.Add_String.add(__caps, __cps_v_136, struct_name, (__cps_v_135) => {
          return __caps.Add_String.add(__caps, __cps_v_135, "<'a> {\n", (s) => {
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
      return emit_token_accessor__lto(__caps, s, label, t, Bool["false"], __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        return list_contains_string__lto(__caps, token_defs, name, (__cps_v_137) => {
          if ((__cps_v_137[LUMO_TAG] === "true")) {
            return emit_token_accessor__lto(__caps, s, label, TokenRef["named"](name), Bool["false"], __k);
          } else if ((__cps_v_137[LUMO_TAG] === "false")) {
            return __k(emit_node_accessor__lto(s, label, name, Bool["false"]));
          } else {
            return __lumo_match_error(__cps_v_137);
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
      return emit_token_accessor__lto(__caps, s, label, t, Bool["true"], __k);
    } else if ((elem[LUMO_TAG] === "node")) {
      const n = elem.args[0];
      if ((n[LUMO_TAG] === "mk")) {
        const name = n.args[0];
        return list_contains_string__lto(__caps, token_defs, name, (__cps_v_138) => {
          if ((__cps_v_138[LUMO_TAG] === "true")) {
            return emit_token_accessor__lto(__caps, s, label, TokenRef["named"](name), Bool["true"], __k);
          } else if ((__cps_v_138[LUMO_TAG] === "false")) {
            return __k(emit_node_accessor__lto(s, label, name, Bool["true"]));
          } else {
            return __lumo_match_error(__cps_v_138);
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
      return keyword_variant__lto(__caps, kw, __k);
    } else {
      return ((t[LUMO_TAG] === "symbol") ? ((sym) => {
        return __k(symbol_variant__lto(sym));
      })(t.args[0]) : __lumo_match_error(t));
    }
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
        return __caps.Add_String.add(__caps, s, "    ", (__cps_v_143) => {
          return __caps.Add_String.add(__caps, __cps_v_143, name, (__cps_v_142) => {
            return __caps.Add_String.add(__caps, __cps_v_142, "(", (__cps_v_141) => {
              return __caps.Add_String.add(__caps, __cps_v_141, name, (__cps_v_140) => {
                return __caps.Add_String.add(__caps, __cps_v_140, "<'a>),\n", (__cps_v_139) => {
                  return emit_enum_variants__lto(__caps, __cps_v_139, rest, __k);
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
        return __caps.Add_String.add(__caps, "            .or_else(|| ", name, (__cps_v_147) => {
          return __caps.Add_String.add(__caps, __cps_v_147, "::cast(node).map(Self::", (__cps_v_146) => {
            return __caps.Add_String.add(__caps, __cps_v_146, name, (__cps_v_145) => {
              return __caps.Add_String.add(__caps, __cps_v_145, "))\n", (line) => {
                return __caps.Add_String.add(__caps, s, line, (__cps_v_144) => {
                  return emit_enum_cast_chain__lto(__caps, __cps_v_144, rest, __k);
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
        return __caps.Add_String.add(__caps, "            Self::", name, (__cps_v_149) => {
          return __caps.Add_String.add(__caps, __cps_v_149, "(n) => n.syntax(),\n", (line) => {
            return __caps.Add_String.add(__caps, s, line, (__cps_v_148) => {
              return emit_enum_syntax_arms__lto(__caps, __cps_v_148, rest, __k);
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
  return run__lto(__caps, __k);
}

export function main() {
  return __trampoline(__main_cps({ IO_IO: IO(__identity), FS_FS: FS(__identity), Process_Process: Process(__identity), StrOps_StrOps: StrOps(__identity), Add_String: __impl_String_Add(__identity), Sub_Number: __impl_Number_Sub(__identity), NumOps_NumOps: NumOps(__identity), Add_Number: __impl_Number_Add(__identity), PartialEq_String: __impl_String_PartialEq(__identity), PartialOrd_Number: __impl_Number_PartialOrd(__identity) }, __identity));
}

export function run_generate(__caps, file, count, syntax_kind_code, ast_code, __k) {
  return __thunk(() => {
    return __caps.Process_Process.args_count(__caps, (__cps_v_152) => {
      return __caps.PartialOrd_Number.cmp(__caps, __cps_v_152, 3, (__cps_v_151) => {
        const __k_119 = (__cps_v_150) => {
          if ((__cps_v_150[LUMO_TAG] === "true")) {
            return __k(write_output__lto(".", file, count, syntax_kind_code, ast_code));
          } else if ((__cps_v_150[LUMO_TAG] === "false")) {
            return __caps.Process_Process.arg_at(__caps, 2, (out_dir) => {
              return __k(write_output__lto(out_dir, file, count, syntax_kind_code, ast_code));
            });
          } else {
            return __lumo_match_error(__cps_v_150);
          }
        };
        if ((__cps_v_151[LUMO_TAG] === "less")) {
          return __k_119(Bool["true"]);
        } else if ((__cps_v_151[LUMO_TAG] === "equal")) {
          return __k_119(Bool["false"]);
        } else {
          return ((__cps_v_151[LUMO_TAG] === "greater") ? __k_119(Bool["false"]) : __lumo_match_error(__cps_v_151));
        }
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
      return list_length_rules__lto(__caps, rest, (__cps_v_153) => {
        return __caps.Add_Number.add(__caps, 1, __cps_v_153, __k);
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
    return __caps.PartialEq_String.eq(__caps, c, " ", (__cps_v_157) => {
      if ((__cps_v_157[LUMO_TAG] === "true")) {
        return __k(Bool["true"]);
      } else if ((__cps_v_157[LUMO_TAG] === "false")) {
        return __caps.PartialEq_String.eq(__caps, c, "\n", (__cps_v_156) => {
          if ((__cps_v_156[LUMO_TAG] === "true")) {
            return __k(Bool["true"]);
          } else if ((__cps_v_156[LUMO_TAG] === "false")) {
            return __caps.PartialEq_String.eq(__caps, c, "\t", (__cps_v_155) => {
              if ((__cps_v_155[LUMO_TAG] === "true")) {
                return __k(Bool["true"]);
              } else if ((__cps_v_155[LUMO_TAG] === "false")) {
                return __caps.PartialEq_String.eq(__caps, c, "\r", (__cps_v_154) => {
                  if ((__cps_v_154[LUMO_TAG] === "true")) {
                    return __k(Bool["true"]);
                  } else if ((__cps_v_154[LUMO_TAG] === "false")) {
                    return __k(Bool["false"]);
                  } else {
                    return __lumo_match_error(__cps_v_154);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_155);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_156);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_157);
      }
    });
  });
}

export function is_alpha(__caps, c, __k) {
  return String.char_code_at(__caps, c, 0, (code) => {
    return __caps.PartialOrd_Number.cmp(__caps, code, 97, (__cps_v_163) => {
      const __k_130 = (__cps_v_162) => {
        if ((__cps_v_162[LUMO_TAG] === "true")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 122, (__cps_v_161) => {
            if ((__cps_v_161[LUMO_TAG] === "less")) {
              return __k(Bool["true"]);
            } else if ((__cps_v_161[LUMO_TAG] === "equal")) {
              return __k(Bool["true"]);
            } else {
              return ((__cps_v_161[LUMO_TAG] === "greater") ? __k(Bool["false"]) : __lumo_match_error(__cps_v_161));
            }
          });
        } else if ((__cps_v_162[LUMO_TAG] === "false")) {
          return __caps.PartialOrd_Number.cmp(__caps, code, 65, (__cps_v_160) => {
            const __k_128 = (__cps_v_159) => {
              if ((__cps_v_159[LUMO_TAG] === "true")) {
                return __caps.PartialOrd_Number.cmp(__caps, code, 90, (__cps_v_158) => {
                  if ((__cps_v_158[LUMO_TAG] === "less")) {
                    return __k(Bool["true"]);
                  } else if ((__cps_v_158[LUMO_TAG] === "equal")) {
                    return __k(Bool["true"]);
                  } else {
                    return ((__cps_v_158[LUMO_TAG] === "greater") ? __k(Bool["false"]) : __lumo_match_error(__cps_v_158));
                  }
                });
              } else if ((__cps_v_159[LUMO_TAG] === "false")) {
                return __k(Bool["false"]);
              } else {
                return __lumo_match_error(__cps_v_159);
              }
            };
            if ((__cps_v_160[LUMO_TAG] === "less")) {
              return __k_128(Bool["false"]);
            } else if ((__cps_v_160[LUMO_TAG] === "equal")) {
              return __k_128(Bool["true"]);
            } else {
              return ((__cps_v_160[LUMO_TAG] === "greater") ? __k_128(Bool["true"]) : __lumo_match_error(__cps_v_160));
            }
          });
        } else {
          return __lumo_match_error(__cps_v_162);
        }
      };
      if ((__cps_v_163[LUMO_TAG] === "less")) {
        return __k_130(Bool["false"]);
      } else if ((__cps_v_163[LUMO_TAG] === "equal")) {
        return __k_130(Bool["true"]);
      } else {
        return ((__cps_v_163[LUMO_TAG] === "greater") ? __k_130(Bool["true"]) : __lumo_match_error(__cps_v_163));
      }
    });
  });
}

export function is_ident_start(__caps, c, __k) {
  return __thunk(() => {
    return __k(is_alpha__lto(c));
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
      return String.len(__caps, src, (__cps_v_165) => {
        return __caps.PartialOrd_Number.cmp(__caps, pos, __cps_v_165, (__cps_v_164) => {
          if ((__cps_v_164[LUMO_TAG] === "less")) {
            return __k(Bool["false"]);
          } else if ((__cps_v_164[LUMO_TAG] === "equal")) {
            return __k(Bool["true"]);
          } else {
            return ((__cps_v_164[LUMO_TAG] === "greater") ? __k(Bool["true"]) : __lumo_match_error(__cps_v_164));
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
      return String.len(__caps, src, (__cps_v_168) => {
        return __caps.PartialOrd_Number.cmp(__caps, pos, __cps_v_168, (__cps_v_167) => {
          const __k_135 = (__cps_v_166) => {
            if ((__cps_v_166[LUMO_TAG] === "true")) {
              return String.char_at(__caps, src, pos, __k);
            } else if ((__cps_v_166[LUMO_TAG] === "false")) {
              return __k("");
            } else {
              return __lumo_match_error(__cps_v_166);
            }
          };
          if ((__cps_v_167[LUMO_TAG] === "less")) {
            return __k_135(Bool["true"]);
          } else if ((__cps_v_167[LUMO_TAG] === "equal")) {
            return __k_135(Bool["false"]);
          } else {
            return ((__cps_v_167[LUMO_TAG] === "greater") ? __k_135(Bool["false"]) : __lumo_match_error(__cps_v_167));
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
      return __caps.Add_Number.add(__caps, pos, n, (__cps_v_169) => {
        return __k(ParseState["mk"](src, __cps_v_169));
      });
    } else {
      return __lumo_match_error(st);
    }
  });
}

export function skip_ws(__caps, st, __k) {
  return __thunk(() => {
    const __match_145 = state_eof__lto(st);
    if ((__match_145[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__match_145[LUMO_TAG] === "false")) {
      const c = state_peek__lto(st);
      const __match_146 = is_whitespace__lto(c);
      if ((__match_146[LUMO_TAG] === "true")) {
        return skip_ws__lto(__caps, state_advance__lto(st, 1), __k);
      } else if ((__match_146[LUMO_TAG] === "false")) {
        return __caps.PartialEq_String.eq(__caps, c, "/", (__cps_v_176) => {
          if ((__cps_v_176[LUMO_TAG] === "true")) {
            return __caps.Add_Number.add(__caps, state_pos(st), 1, (next_pos) => {
              return String.len(__caps, state_src(st), (__cps_v_175) => {
                return __caps.PartialOrd_Number.cmp(__caps, next_pos, __cps_v_175, (__cps_v_174) => {
                  const __k_142 = (__cps_v_173) => {
                    if ((__cps_v_173[LUMO_TAG] === "true")) {
                      return String.char_at(__caps, state_src(st), next_pos, (__cps_v_172) => {
                        return __caps.PartialEq_String.eq(__caps, __cps_v_172, "/", (__cps_v_171) => {
                          if ((__cps_v_171[LUMO_TAG] === "true")) {
                            return skip_line__lto(__caps, state_advance__lto(st, 2), (__cps_v_170) => {
                              return skip_ws__lto(__caps, __cps_v_170, __k);
                            });
                          } else if ((__cps_v_171[LUMO_TAG] === "false")) {
                            return __k(st);
                          } else {
                            return __lumo_match_error(__cps_v_171);
                          }
                        });
                      });
                    } else if ((__cps_v_173[LUMO_TAG] === "false")) {
                      return __k(st);
                    } else {
                      return __lumo_match_error(__cps_v_173);
                    }
                  };
                  if ((__cps_v_174[LUMO_TAG] === "less")) {
                    return __k_142(Bool["true"]);
                  } else if ((__cps_v_174[LUMO_TAG] === "equal")) {
                    return __k_142(Bool["false"]);
                  } else {
                    return ((__cps_v_174[LUMO_TAG] === "greater") ? __k_142(Bool["false"]) : __lumo_match_error(__cps_v_174));
                  }
                });
              });
            });
          } else if ((__cps_v_176[LUMO_TAG] === "false")) {
            return __k(st);
          } else {
            return __lumo_match_error(__cps_v_176);
          }
        });
      } else {
        return __lumo_match_error(__match_146);
      }
    } else {
      return __lumo_match_error(__match_145);
    }
  });
}

export function skip_line(__caps, st, __k) {
  return __thunk(() => {
    const __match_151 = state_eof__lto(st);
    if ((__match_151[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__match_151[LUMO_TAG] === "false")) {
      return __caps.PartialEq_String.eq(__caps, state_peek__lto(st), "\n", (__cps_v_177) => {
        if ((__cps_v_177[LUMO_TAG] === "true")) {
          return __k(state_advance__lto(st, 1));
        } else if ((__cps_v_177[LUMO_TAG] === "false")) {
          return skip_line__lto(__caps, state_advance__lto(st, 1), __k);
        } else {
          return __lumo_match_error(__cps_v_177);
        }
      });
    } else {
      return __lumo_match_error(__match_151);
    }
  });
}

export function parse_ident(__caps, st, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    const __match_153 = state_eof__lto(st2);
    if ((__match_153[LUMO_TAG] === "true")) {
      return __k(ParseResult["err"]("expected identifier, got EOF", state_pos(st2)));
    } else if ((__match_153[LUMO_TAG] === "false")) {
      return is_ident_start(__caps, state_peek__lto(st2), (__cps_v_181) => {
        if ((__cps_v_181[LUMO_TAG] === "true")) {
          const start = state_pos(st2);
          return scan_ident_rest(__caps, state_advance__lto(st2, 1), (end_st) => {
            const end_pos = state_pos(end_st);
            return String.slice(__caps, state_src(st2), start, end_pos, (__cps_v_180) => {
              return __k(ParseResult["ok"](__cps_v_180, end_st));
            });
          });
        } else if ((__cps_v_181[LUMO_TAG] === "false")) {
          return __caps.Add_String.add(__caps, "expected identifier, got '", state_peek__lto(st2), (__cps_v_179) => {
            return __caps.Add_String.add(__caps, __cps_v_179, "'", (__cps_v_178) => {
              return __k(ParseResult["err"](__cps_v_178, state_pos(st2)));
            });
          });
        } else {
          return __lumo_match_error(__cps_v_181);
        }
      });
    } else {
      return __lumo_match_error(__match_153);
    }
  });
}

export function scan_ident_rest(__caps, st, __k) {
  return __thunk(() => {
    const __match_155 = state_eof__lto(st);
    if ((__match_155[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__match_155[LUMO_TAG] === "false")) {
      return is_ident_continue__lto(__caps, state_peek__lto(st), (__cps_v_182) => {
        if ((__cps_v_182[LUMO_TAG] === "true")) {
          return scan_ident_rest(__caps, state_advance__lto(st, 1), __k);
        } else if ((__cps_v_182[LUMO_TAG] === "false")) {
          return __k(st);
        } else {
          return __lumo_match_error(__cps_v_182);
        }
      });
    } else {
      return __lumo_match_error(__match_155);
    }
  });
}

export function expect(__caps, st, expected, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    return String.len(__caps, expected, (len) => {
      const src = state_src(st2);
      const pos = state_pos(st2);
      return String.len(__caps, src, (__cps_v_193) => {
        return __caps.Sub_Number.sub(__caps, __cps_v_193, pos, (remaining) => {
          return __caps.PartialOrd_Number.cmp(__caps, remaining, len, (__cps_v_192) => {
            const __k_151 = (__cps_v_191) => {
              if ((__cps_v_191[LUMO_TAG] === "true")) {
                return __caps.Add_Number.add(__caps, pos, len, (__cps_v_190) => {
                  return String.slice(__caps, src, pos, __cps_v_190, (slice) => {
                    return __caps.PartialEq_String.eq(__caps, slice, expected, (__cps_v_189) => {
                      if ((__cps_v_189[LUMO_TAG] === "true")) {
                        return __k(ParseResult["ok"](expected, state_advance__lto(st2, len)));
                      } else if ((__cps_v_189[LUMO_TAG] === "false")) {
                        return __caps.Add_String.add(__caps, "expected '", expected, (__cps_v_188) => {
                          return __caps.Add_String.add(__caps, __cps_v_188, "', got '", (__cps_v_187) => {
                            return __caps.Add_String.add(__caps, __cps_v_187, slice, (__cps_v_186) => {
                              return __caps.Add_String.add(__caps, __cps_v_186, "'", (__cps_v_185) => {
                                return __k(ParseResult["err"](__cps_v_185, pos));
                              });
                            });
                          });
                        });
                      } else {
                        return __lumo_match_error(__cps_v_189);
                      }
                    });
                  });
                });
              } else if ((__cps_v_191[LUMO_TAG] === "false")) {
                return __caps.Add_String.add(__caps, "expected '", expected, (__cps_v_184) => {
                  return __caps.Add_String.add(__caps, __cps_v_184, "'", (__cps_v_183) => {
                    return __k(ParseResult["err"](__cps_v_183, pos));
                  });
                });
              } else {
                return __lumo_match_error(__cps_v_191);
              }
            };
            if ((__cps_v_192[LUMO_TAG] === "less")) {
              return __k_151(Bool["false"]);
            } else if ((__cps_v_192[LUMO_TAG] === "equal")) {
              return __k_151(Bool["true"]);
            } else {
              return ((__cps_v_192[LUMO_TAG] === "greater") ? __k_151(Bool["true"]) : __lumo_match_error(__cps_v_192));
            }
          });
        });
      });
    });
  });
}

export function parse_quoted(__caps, st, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    return __caps.PartialEq_String.eq(__caps, state_peek__lto(st2), "'", (__cps_v_194) => {
      if ((__cps_v_194[LUMO_TAG] === "true")) {
        return __caps.Add_Number.add(__caps, state_pos(st2), 1, (start) => {
          return scan_until_quote__lto(__caps, state_advance__lto(st2, 1), (end_st) => {
            const end_pos = state_pos(end_st);
            return String.slice(__caps, state_src(st2), start, end_pos, (content) => {
              return __k(ParseResult["ok"](content, state_advance__lto(end_st, 1)));
            });
          });
        });
      } else if ((__cps_v_194[LUMO_TAG] === "false")) {
        return __k(ParseResult["err"]("expected quoted literal", state_pos(st2)));
      } else {
        return __lumo_match_error(__cps_v_194);
      }
    });
  });
}

export function scan_until_quote(__caps, st, __k) {
  return __thunk(() => {
    const __match_161 = state_eof__lto(st);
    if ((__match_161[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__match_161[LUMO_TAG] === "false")) {
      return __caps.PartialEq_String.eq(__caps, state_peek__lto(st), "'", (__cps_v_195) => {
        if ((__cps_v_195[LUMO_TAG] === "true")) {
          return __k(st);
        } else if ((__cps_v_195[LUMO_TAG] === "false")) {
          return scan_until_quote__lto(__caps, state_advance__lto(st, 1), __k);
        } else {
          return __lumo_match_error(__cps_v_195);
        }
      });
    } else {
      return __lumo_match_error(__match_161);
    }
  });
}

export function peek_char(__caps, st, __k) {
  return skip_ws__lto(__caps, st, (__cps_v_196) => {
    return __k(state_peek__lto(__cps_v_196));
  });
}

export function peek_is_rule_start(__caps, st, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    return is_ident_start(__caps, state_peek__lto(st2), (__cps_v_197) => {
      if ((__cps_v_197[LUMO_TAG] === "true")) {
        return scan_ident_rest(__caps, state_advance__lto(st2, 1), (st3) => {
          return skip_ws__lto(__caps, st3, (st4) => {
            return __caps.PartialEq_String.eq(__caps, state_peek__lto(st4), "=", __k);
          });
        });
      } else if ((__cps_v_197[LUMO_TAG] === "false")) {
        return __k(Bool["false"]);
      } else {
        return __lumo_match_error(__cps_v_197);
      }
    });
  });
}

export function classify_literal(__caps, text, __k) {
  return has_alpha__lto(__caps, text, 0, (__cps_v_198) => {
    if ((__cps_v_198[LUMO_TAG] === "true")) {
      return __k(TokenRef["keyword"](text));
    } else if ((__cps_v_198[LUMO_TAG] === "false")) {
      return __k(TokenRef["symbol"](text));
    } else {
      return __lumo_match_error(__cps_v_198);
    }
  });
}

export function has_alpha(__caps, s, i, __k) {
  return String.len(__caps, s, (__cps_v_204) => {
    return __caps.PartialOrd_Number.cmp(__caps, i, __cps_v_204, (__cps_v_203) => {
      const __k_159 = (__cps_v_202) => {
        if ((__cps_v_202[LUMO_TAG] === "true")) {
          return __k(Bool["false"]);
        } else if ((__cps_v_202[LUMO_TAG] === "false")) {
          return String.char_at(__caps, s, i, (__cps_v_201) => {
            const __cps_v_200 = is_alpha__lto(__cps_v_201);
            if ((__cps_v_200[LUMO_TAG] === "true")) {
              return __k(Bool["true"]);
            } else if ((__cps_v_200[LUMO_TAG] === "false")) {
              return __caps.Add_Number.add(__caps, i, 1, (__cps_v_199) => {
                return has_alpha__lto(__caps, s, __cps_v_199, __k);
              });
            } else {
              return __lumo_match_error(__cps_v_200);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_202);
        }
      };
      if ((__cps_v_203[LUMO_TAG] === "less")) {
        return __k_159(Bool["false"]);
      } else if ((__cps_v_203[LUMO_TAG] === "equal")) {
        return __k_159(Bool["true"]);
      } else {
        return ((__cps_v_203[LUMO_TAG] === "greater") ? __k_159(Bool["true"]) : __lumo_match_error(__cps_v_203));
      }
    });
  });
}

export function parse_grammar(__caps, src, __k) {
  return __thunk(() => {
    const st = ParseState["mk"](src, 0);
    return parse_grammar_items__lto(__caps, st, List["nil"], List["nil"], __k);
  });
}

export function parse_grammar_items(__caps, st, tokens, rules, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    const __match_168 = state_eof__lto(st2);
    if ((__match_168[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_rule(rules)), st2));
    } else if ((__match_168[LUMO_TAG] === "false")) {
      return __caps.PartialEq_String.eq(__caps, state_peek__lto(st2), "@", (__cps_v_207) => {
        if ((__cps_v_207[LUMO_TAG] === "true")) {
          return parse_token_def(__caps, st2, (__cps_v_206) => {
            if ((__cps_v_206[LUMO_TAG] === "ok")) {
              const new_tokens = __cps_v_206.args[0];
              const st3 = __cps_v_206.args[1];
              return parse_grammar_items__lto(__caps, st3, list_concat_string(new_tokens, tokens), rules, __k);
            } else if ((__cps_v_206[LUMO_TAG] === "err")) {
              const msg = __cps_v_206.args[0];
              const pos = __cps_v_206.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_206);
            }
          });
        } else if ((__cps_v_207[LUMO_TAG] === "false")) {
          return parse_rule(__caps, st2, (__cps_v_205) => {
            if ((__cps_v_205[LUMO_TAG] === "ok")) {
              const rule = __cps_v_205.args[0];
              const st3 = __cps_v_205.args[1];
              return parse_grammar_items__lto(__caps, st3, tokens, List["cons"](rule, rules), __k);
            } else if ((__cps_v_205[LUMO_TAG] === "err")) {
              const msg = __cps_v_205.args[0];
              const pos = __cps_v_205.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_205);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_207);
        }
      });
    } else {
      return __lumo_match_error(__match_168);
    }
  });
}

export function parse_token_def(__caps, st, __k) {
  return expect__lto(__caps, st, "@token", (__cps_v_208) => {
    if ((__cps_v_208[LUMO_TAG] === "err")) {
      const msg = __cps_v_208.args[0];
      const pos = __cps_v_208.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_208[LUMO_TAG] === "ok")) {
      const st2 = __cps_v_208.args[1];
      return parse_token_names__lto(__caps, st2, List["nil"], __k);
    } else {
      return __lumo_match_error(__cps_v_208);
    }
  });
}

export function parse_token_names(__caps, st, acc, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    const __match_173 = state_eof__lto(st2);
    if ((__match_173[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
    } else if ((__match_173[LUMO_TAG] === "false")) {
      return peek_is_rule_start__lto(__caps, st2, (__cps_v_212) => {
        if ((__cps_v_212[LUMO_TAG] === "true")) {
          return __k(ParseResult["ok"](list_reverse_string(acc), st2));
        } else if ((__cps_v_212[LUMO_TAG] === "false")) {
          return __caps.PartialEq_String.eq(__caps, state_peek__lto(st2), "@", (__cps_v_211) => {
            if ((__cps_v_211[LUMO_TAG] === "true")) {
              return __k(ParseResult["ok"](list_reverse_string(acc), st2));
            } else if ((__cps_v_211[LUMO_TAG] === "false")) {
              return is_ident_start(__caps, state_peek__lto(st2), (__cps_v_210) => {
                if ((__cps_v_210[LUMO_TAG] === "true")) {
                  return parse_ident__lto(__caps, st2, (__cps_v_209) => {
                    if ((__cps_v_209[LUMO_TAG] === "ok")) {
                      const name = __cps_v_209.args[0];
                      const st3 = __cps_v_209.args[1];
                      return parse_token_names__lto(__caps, st3, List["cons"](name, acc), __k);
                    } else if ((__cps_v_209[LUMO_TAG] === "err")) {
                      const msg = __cps_v_209.args[0];
                      const pos = __cps_v_209.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_209);
                    }
                  });
                } else if ((__cps_v_210[LUMO_TAG] === "false")) {
                  return __k(ParseResult["ok"](list_reverse_string(acc), st2));
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
      return __lumo_match_error(__match_173);
    }
  });
}

export function parse_rule(__caps, st, __k) {
  return parse_ident__lto(__caps, st, (__cps_v_215) => {
    if ((__cps_v_215[LUMO_TAG] === "err")) {
      const msg = __cps_v_215.args[0];
      const pos = __cps_v_215.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_215[LUMO_TAG] === "ok")) {
      const name = __cps_v_215.args[0];
      const st2 = __cps_v_215.args[1];
      return expect__lto(__caps, st2, "=", (__cps_v_214) => {
        if ((__cps_v_214[LUMO_TAG] === "err")) {
          const msg = __cps_v_214.args[0];
          const pos = __cps_v_214.args[1];
          return __k(ParseResult["err"](msg, pos));
        } else if ((__cps_v_214[LUMO_TAG] === "ok")) {
          const st3 = __cps_v_214.args[1];
          return parse_rule_body__lto(__caps, st3, name, (__cps_v_213) => {
            if ((__cps_v_213[LUMO_TAG] === "err")) {
              const msg = __cps_v_213.args[0];
              const pos = __cps_v_213.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else if ((__cps_v_213[LUMO_TAG] === "ok")) {
              const body = __cps_v_213.args[0];
              const st4 = __cps_v_213.args[1];
              return __k(ParseResult["ok"](Rule["mk"](name, body), st4));
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
}

export function parse_alternatives(__caps, st, __k) {
  return parse_alt_items__lto(__caps, st, List["nil"], __k);
}

export function parse_alt_items(__caps, st, acc, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    return peek_char(__caps, st2, (__cps_v_221) => {
      return __caps.PartialEq_String.eq(__caps, __cps_v_221, "|", (__cps_v_220) => {
        if ((__cps_v_220[LUMO_TAG] === "true")) {
          return skip_ws__lto(__caps, st2, (__cps_v_219) => {
            const st3 = state_advance__lto(__cps_v_219, 1);
            return skip_ws__lto(__caps, st3, (st4) => {
              return __caps.PartialEq_String.eq(__caps, state_peek__lto(st4), "'", (__cps_v_218) => {
                if ((__cps_v_218[LUMO_TAG] === "true")) {
                  return parse_quoted__lto(__caps, st4, (__cps_v_217) => {
                    if ((__cps_v_217[LUMO_TAG] === "ok")) {
                      const lit = __cps_v_217.args[0];
                      const st5 = __cps_v_217.args[1];
                      return parse_alt_items__lto(__caps, st5, List["cons"](Alternative["mk"](lit), acc), __k);
                    } else if ((__cps_v_217[LUMO_TAG] === "err")) {
                      const msg = __cps_v_217.args[0];
                      const pos = __cps_v_217.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_217);
                    }
                  });
                } else if ((__cps_v_218[LUMO_TAG] === "false")) {
                  return parse_ident__lto(__caps, st3, (__cps_v_216) => {
                    if ((__cps_v_216[LUMO_TAG] === "ok")) {
                      const name = __cps_v_216.args[0];
                      const st5 = __cps_v_216.args[1];
                      return parse_alt_items__lto(__caps, st5, List["cons"](Alternative["mk"](name), acc), __k);
                    } else if ((__cps_v_216[LUMO_TAG] === "err")) {
                      const msg = __cps_v_216.args[0];
                      const pos = __cps_v_216.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_216);
                    }
                  });
                } else {
                  return __lumo_match_error(__cps_v_218);
                }
              });
            });
          });
        } else if ((__cps_v_220[LUMO_TAG] === "false")) {
          return __k(ParseResult["ok"](RuleBody["alternatives"](list_reverse_alt(acc)), st2));
        } else {
          return __lumo_match_error(__cps_v_220);
        }
      });
    });
  });
}

export function parse_sequence(__caps, st, __k) {
  return parse_seq_elements(__caps, st, List["nil"], __k);
}

export function parse_seq_elements(__caps, st, acc, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    const __match_185 = state_eof__lto(st2);
    if ((__match_185[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
    } else if ((__match_185[LUMO_TAG] === "false")) {
      return is_seq_terminator__lto(__caps, st2, (__cps_v_223) => {
        if ((__cps_v_223[LUMO_TAG] === "true")) {
          return __k(ParseResult["ok"](RuleBody["sequence"](list_reverse_elem(acc)), st2));
        } else if ((__cps_v_223[LUMO_TAG] === "false")) {
          return parse_element(__caps, st2, (__cps_v_222) => {
            if ((__cps_v_222[LUMO_TAG] === "ok")) {
              const elem = __cps_v_222.args[0];
              const st3 = __cps_v_222.args[1];
              return parse_seq_elements(__caps, st3, List["cons"](elem, acc), __k);
            } else if ((__cps_v_222[LUMO_TAG] === "err")) {
              const msg = __cps_v_222.args[0];
              const pos = __cps_v_222.args[1];
              return __k(ParseResult["err"](msg, pos));
            } else {
              return __lumo_match_error(__cps_v_222);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_223);
        }
      });
    } else {
      return __lumo_match_error(__match_185);
    }
  });
}

export function parse_element(__caps, st, __k) {
  return parse_atom__lto(__caps, st, (__cps_v_224) => {
    if ((__cps_v_224[LUMO_TAG] === "err")) {
      const msg = __cps_v_224.args[0];
      const pos = __cps_v_224.args[1];
      return __k(ParseResult["err"](msg, pos));
    } else if ((__cps_v_224[LUMO_TAG] === "ok")) {
      const elem = __cps_v_224.args[0];
      const st2 = __cps_v_224.args[1];
      return apply_postfix_elem__lto(__caps, elem, st2, __k);
    } else {
      return __lumo_match_error(__cps_v_224);
    }
  });
}

export function apply_postfix_elem(__caps, elem, st, __k) {
  return __thunk(() => {
    const __match_189 = state_eof__lto(st);
    if ((__match_189[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](elem, st));
    } else if ((__match_189[LUMO_TAG] === "false")) {
      return __caps.PartialEq_String.eq(__caps, state_peek__lto(st), "?", (__cps_v_226) => {
        if ((__cps_v_226[LUMO_TAG] === "true")) {
          return apply_postfix_elem__lto(__caps, Element["optional"](elem), state_advance__lto(st, 1), __k);
        } else if ((__cps_v_226[LUMO_TAG] === "false")) {
          return __caps.PartialEq_String.eq(__caps, state_peek__lto(st), "*", (__cps_v_225) => {
            if ((__cps_v_225[LUMO_TAG] === "true")) {
              return apply_postfix_elem__lto(__caps, Element["repeated"](elem), state_advance__lto(st, 1), __k);
            } else if ((__cps_v_225[LUMO_TAG] === "false")) {
              return __k(ParseResult["ok"](elem, st));
            } else {
              return __lumo_match_error(__cps_v_225);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_226);
        }
      });
    } else {
      return __lumo_match_error(__match_189);
    }
  });
}

export function parse_group_elements(__caps, st, acc, __k) {
  return skip_ws__lto(__caps, st, (st2) => {
    return __caps.PartialEq_String.eq(__caps, state_peek__lto(st2), ")", (__cps_v_228) => {
      if ((__cps_v_228[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
      } else if ((__cps_v_228[LUMO_TAG] === "false")) {
        return parse_element(__caps, st2, (__cps_v_227) => {
          if ((__cps_v_227[LUMO_TAG] === "ok")) {
            const elem = __cps_v_227.args[0];
            const st3 = __cps_v_227.args[1];
            return parse_group_elements__lto(__caps, st3, List["cons"](elem, acc), __k);
          } else if ((__cps_v_227[LUMO_TAG] === "err")) {
            const msg = __cps_v_227.args[0];
            const pos = __cps_v_227.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_227);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_228);
      }
    });
  });
}

export function resolve_grammar(__caps, g, __k) {
  return __thunk(() => {
    if ((g[LUMO_TAG] === "mk")) {
      const token_defs = g.args[0];
      const rules = g.args[1];
      return resolve_rules(__caps, token_defs, rules, (__cps_v_229) => {
        return __k(Grammar["mk"](token_defs, __cps_v_229));
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
          return resolve_rules(__caps, token_defs, rest, (__cps_v_230) => {
            return __k(List["cons"](Rule["mk"](name, resolved_body), __cps_v_230));
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
      return resolve_elements(__caps, token_defs, elems, (__cps_v_231) => {
        return __k(RuleBody["sequence"](__cps_v_231));
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
      return resolve_element(__caps, token_defs, elem, (__cps_v_232) => {
        return resolve_elements(__caps, token_defs, rest, (__cps_v_233) => {
          return __k(List["cons"](__cps_v_232, __cps_v_233));
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
        return list_contains_string__lto(__caps, token_defs, name, (__cps_v_238) => {
          if ((__cps_v_238[LUMO_TAG] === "true")) {
            return __k(Element["token"](TokenRef["named"](name)));
          } else if ((__cps_v_238[LUMO_TAG] === "false")) {
            return __k(elem);
          } else {
            return __lumo_match_error(__cps_v_238);
          }
        });
      } else {
        return __lumo_match_error(ref);
      }
    } else {
      return ((elem[LUMO_TAG] === "labeled") ? ((label) => {
        const inner = elem.args[1];
        return resolve_element(__caps, token_defs, inner, (__cps_v_237) => {
          return __k(Element["labeled"](label, __cps_v_237));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "optional") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_236) => {
          return __k(Element["optional"](__cps_v_236));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "repeated") ? ((inner) => {
        return resolve_element(__caps, token_defs, inner, (__cps_v_235) => {
          return __k(Element["repeated"](__cps_v_235));
        });
      })(elem.args[0]) : ((elem[LUMO_TAG] === "group") ? ((elems) => {
        return resolve_elements(__caps, token_defs, elems, (__cps_v_234) => {
          return __k(Element["group"](__cps_v_234));
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
      return __caps.PartialEq_String.eq(__caps, x, target, (__cps_v_239) => {
        if ((__cps_v_239[LUMO_TAG] === "true")) {
          return __k(Bool["true"]);
        } else if ((__cps_v_239[LUMO_TAG] === "false")) {
          return list_contains_string__lto(__caps, rest, target, __k);
        } else {
          return __lumo_match_error(__cps_v_239);
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


export const __impl_String_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.concat(__caps, self, other, (__cps_v_240) => {
        return __k_handle(__k_perform(__cps_v_240));
      });
    });
  } };
};

export const __impl_String_PartialEq = (__k_handle) => {
  return { eq: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.StrOps_StrOps.eq(__caps, self, other, (__cps_v_241) => {
        return __k_handle(__k_perform(__cps_v_241));
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
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
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
      return __k_handle(__k_perform(((__match_209) => {
        if ((__match_209[LUMO_TAG] === "true")) {
          return Bool["false"];
        } else if ((__match_209[LUMO_TAG] === "false")) {
          return Bool["true"];
        } else {
          return __lumo_match_error(__match_209);
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
      return __caps.NumOps_NumOps.eq(__caps, self, other, (__cps_v_242) => {
        return __k_handle(__k_perform(__cps_v_242));
      });
    });
  } };
};

export const __impl_Number_PartialOrd = (__k_handle) => {
  return { cmp: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.cmp(__caps, self, other, (__cps_v_243) => {
        return __k_handle(__k_perform(__cps_v_243));
      });
    });
  } };
};

export const __impl_Number_Add = (__k_handle) => {
  return { add: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.add(__caps, self, other, (__cps_v_244) => {
        return __k_handle(__k_perform(__cps_v_244));
      });
    });
  } };
};

export const __impl_Number_Sub = (__k_handle) => {
  return { sub: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.sub(__caps, self, other, (__cps_v_245) => {
        return __k_handle(__k_perform(__cps_v_245));
      });
    });
  } };
};

export const __impl_Number_Mul = (__k_handle) => {
  return { mul: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mul(__caps, self, other, (__cps_v_246) => {
        return __k_handle(__k_perform(__cps_v_246));
      });
    });
  } };
};

export const __impl_Number_Div = (__k_handle) => {
  return { div: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.div(__caps, self, other, (__cps_v_247) => {
        return __k_handle(__k_perform(__cps_v_247));
      });
    });
  } };
};

export const __impl_Number_Mod = (__k_handle) => {
  return { mod_: (__caps, self, other, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.mod_(__caps, self, other, (__cps_v_248) => {
        return __k_handle(__k_perform(__cps_v_248));
      });
    });
  } };
};

export const __impl_Number_Neg = (__k_handle) => {
  return { neg: (__caps, self, __k_perform) => {
    return __thunk(() => {
      return __caps.NumOps_NumOps.neg(__caps, self, (__cps_v_249) => {
        return __k_handle(__k_perform(__cps_v_249));
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
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })(a, b)));
    });
  }, cmp: (__caps, a, b, __k_perform) => {
    return __thunk(() => {
      return __k_handle(__k_perform(((__match_210) => {
        if ((__match_210[LUMO_TAG] === "true")) {
          return Ordering["less"];
        } else if ((__match_210[LUMO_TAG] === "false")) {
          const __match_211 = ((a, b) => {
            if ((a === b)) {
              return Bool["true"];
            } else {
              return Bool["false"];
            }
          })(a, b);
          if ((__match_211[LUMO_TAG] === "true")) {
            return Ordering["equal"];
          } else if ((__match_211[LUMO_TAG] === "false")) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_211);
          }
        } else {
          return __lumo_match_error(__match_210);
        }
      })(((a, b) => {
        if ((a < b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
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

export function to_screaming_snake_loop__lto(__caps, name, i, acc, __k) {
  return __thunk(() => {
    return String.len(__caps, name, (__lto_other_1) => {
      const __k_208 = (__cps_v_257) => {
        const __k_207 = (__cps_v_256) => {
          if ((__cps_v_256[LUMO_TAG] === "true")) {
            return __k(acc);
          } else if ((__cps_v_256[LUMO_TAG] === "false")) {
            return String.char_at(__caps, name, i, (c) => {
              return String.char_code_at(__caps, c, 0, (code) => {
                const __k_205 = (is_upper) => {
                  if ((is_upper[LUMO_TAG] === "true")) {
                    let __match_214;
                    let __match_233;
                    const __lto_self_12 = 0;
                    __match_233 = ((__lto_other_13) => {
                      let __match_231;
                      const a = __lto_self_12;
                      const b = __lto_other_13;
                      __match_231 = ((a < b) ? Bool["true"] : Bool["false"]);
                      if ((__match_231[LUMO_TAG] === "true")) {
                        return Ordering["less"];
                      } else if ((__match_231[LUMO_TAG] === "false")) {
                        let __match_232;
                        const a = __lto_self_12;
                        const b = __lto_other_13;
                        if ((a === b)) {
                          __match_232 = Bool["true"];
                        } else {
                          __match_232 = Bool["false"];
                        }
                        if ((__match_232[LUMO_TAG] === "true")) {
                          return Ordering["equal"];
                        } else if ((__match_232[LUMO_TAG] === "false")) {
                          return Ordering["greater"];
                        } else {
                          return __lumo_match_error(__match_232);
                        }
                      } else {
                        return __lumo_match_error(__match_231);
                      }
                    })(i);
                    __match_214 = ((__match_233[LUMO_TAG] === "less") ? Bool["true"] : ((__match_233[LUMO_TAG] === "equal") ? Bool["false"] : ((__match_233[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_233))));
                    if ((__match_214[LUMO_TAG] === "true")) {
                      return String.char_at(__caps, name, ((__lto_self_16) => {
                        const __lto_other_17 = 1;
                        const a = __lto_self_16;
                        const b = __lto_other_17;
                        return (a - b);
                      })(i), (__cps_v_255) => {
                        return String.char_code_at(__caps, __cps_v_255, 0, (prev_code) => {
                          const __k_203 = (prev_lower) => {
                            const __k_201 = (prev_digit) => {
                              if ((prev_lower[LUMO_TAG] === "true")) {
                                const __lto_other_43 = "_";
                                let __lto_self_40;
                                const a = acc;
                                const b = __lto_other_43;
                                __lto_self_40 = (a + b);
                                return to_upper_char(__caps, c, (__lto_other_41) => {
                                  let __cps_v_254;
                                  const a = __lto_self_40;
                                  const b = __lto_other_41;
                                  __cps_v_254 = (a + b);
                                  return to_screaming_snake_loop(__caps, name, ((__lto_self_36) => {
                                    const __lto_other_37 = 1;
                                    const a = __lto_self_36;
                                    const b = __lto_other_37;
                                    return (a + b);
                                  })(i), __cps_v_254, __k);
                                });
                              } else if ((prev_lower[LUMO_TAG] === "false")) {
                                if ((prev_digit[LUMO_TAG] === "true")) {
                                  const __lto_other_55 = "_";
                                  let __lto_self_52;
                                  const a = acc;
                                  const b = __lto_other_55;
                                  __lto_self_52 = (a + b);
                                  return to_upper_char(__caps, c, (__lto_other_53) => {
                                    let __cps_v_253;
                                    const a = __lto_self_52;
                                    const b = __lto_other_53;
                                    __cps_v_253 = (a + b);
                                    return to_screaming_snake_loop(__caps, name, ((__lto_self_48) => {
                                      const __lto_other_49 = 1;
                                      const a = __lto_self_48;
                                      const b = __lto_other_49;
                                      return (a + b);
                                    })(i), __cps_v_253, __k);
                                  });
                                } else if ((prev_digit[LUMO_TAG] === "false")) {
                                  return to_upper_char(__caps, c, (__lto_other_65) => {
                                    let __cps_v_252;
                                    const a = acc;
                                    const b = __lto_other_65;
                                    __cps_v_252 = (a + b);
                                    return to_screaming_snake_loop(__caps, name, ((__lto_self_60) => {
                                      const __lto_other_61 = 1;
                                      const a = __lto_self_60;
                                      const b = __lto_other_61;
                                      return (a + b);
                                    })(i), __cps_v_252, __k);
                                  });
                                } else {
                                  return __lumo_match_error(prev_digit);
                                }
                              } else {
                                return __lumo_match_error(prev_lower);
                              }
                            };
                            let __match_217;
                            let __match_223;
                            __match_223 = ((__lto_other_29) => {
                              let __match_221;
                              const a = prev_code;
                              const b = __lto_other_29;
                              __match_221 = ((a < b) ? Bool["true"] : Bool["false"]);
                              if ((__match_221[LUMO_TAG] === "true")) {
                                return Ordering["less"];
                              } else if ((__match_221[LUMO_TAG] === "false")) {
                                let __match_222;
                                const a = prev_code;
                                const b = __lto_other_29;
                                if ((a === b)) {
                                  __match_222 = Bool["true"];
                                } else {
                                  __match_222 = Bool["false"];
                                }
                                if ((__match_222[LUMO_TAG] === "true")) {
                                  return Ordering["equal"];
                                } else if ((__match_222[LUMO_TAG] === "false")) {
                                  return Ordering["greater"];
                                } else {
                                  return __lumo_match_error(__match_222);
                                }
                              } else {
                                return __lumo_match_error(__match_221);
                              }
                            })(48);
                            __match_217 = ((__match_223[LUMO_TAG] === "less") ? Bool["false"] : ((__match_223[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_223[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_223))));
                            if ((__match_217[LUMO_TAG] === "true")) {
                              let __match_218;
                              __match_218 = ((__lto_other_33) => {
                                let __match_219;
                                const a = prev_code;
                                const b = __lto_other_33;
                                __match_219 = ((a < b) ? Bool["true"] : Bool["false"]);
                                if ((__match_219[LUMO_TAG] === "true")) {
                                  return Ordering["less"];
                                } else if ((__match_219[LUMO_TAG] === "false")) {
                                  let __match_220;
                                  const a = prev_code;
                                  const b = __lto_other_33;
                                  if ((a === b)) {
                                    __match_220 = Bool["true"];
                                  } else {
                                    __match_220 = Bool["false"];
                                  }
                                  if ((__match_220[LUMO_TAG] === "true")) {
                                    return Ordering["equal"];
                                  } else if ((__match_220[LUMO_TAG] === "false")) {
                                    return Ordering["greater"];
                                  } else {
                                    return __lumo_match_error(__match_220);
                                  }
                                } else {
                                  return __lumo_match_error(__match_219);
                                }
                              })(57);
                              if ((__match_218[LUMO_TAG] === "less")) {
                                return __k_201(Bool["true"]);
                              } else if ((__match_218[LUMO_TAG] === "equal")) {
                                return __k_201(Bool["true"]);
                              } else {
                                return ((__match_218[LUMO_TAG] === "greater") ? __k_201(Bool["false"]) : __lumo_match_error(__match_218));
                              }
                            } else if ((__match_217[LUMO_TAG] === "false")) {
                              return __k_201(Bool["false"]);
                            } else {
                              return __lumo_match_error(__match_217);
                            }
                          };
                          let __match_224;
                          let __match_230;
                          __match_230 = ((__lto_other_21) => {
                            let __match_228;
                            const a = prev_code;
                            const b = __lto_other_21;
                            __match_228 = ((a < b) ? Bool["true"] : Bool["false"]);
                            if ((__match_228[LUMO_TAG] === "true")) {
                              return Ordering["less"];
                            } else if ((__match_228[LUMO_TAG] === "false")) {
                              let __match_229;
                              const a = prev_code;
                              const b = __lto_other_21;
                              if ((a === b)) {
                                __match_229 = Bool["true"];
                              } else {
                                __match_229 = Bool["false"];
                              }
                              if ((__match_229[LUMO_TAG] === "true")) {
                                return Ordering["equal"];
                              } else if ((__match_229[LUMO_TAG] === "false")) {
                                return Ordering["greater"];
                              } else {
                                return __lumo_match_error(__match_229);
                              }
                            } else {
                              return __lumo_match_error(__match_228);
                            }
                          })(97);
                          __match_224 = ((__match_230[LUMO_TAG] === "less") ? Bool["false"] : ((__match_230[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_230[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_230))));
                          if ((__match_224[LUMO_TAG] === "true")) {
                            let __match_225;
                            __match_225 = ((__lto_other_25) => {
                              let __match_226;
                              const a = prev_code;
                              const b = __lto_other_25;
                              __match_226 = ((a < b) ? Bool["true"] : Bool["false"]);
                              if ((__match_226[LUMO_TAG] === "true")) {
                                return Ordering["less"];
                              } else if ((__match_226[LUMO_TAG] === "false")) {
                                let __match_227;
                                const a = prev_code;
                                const b = __lto_other_25;
                                if ((a === b)) {
                                  __match_227 = Bool["true"];
                                } else {
                                  __match_227 = Bool["false"];
                                }
                                if ((__match_227[LUMO_TAG] === "true")) {
                                  return Ordering["equal"];
                                } else if ((__match_227[LUMO_TAG] === "false")) {
                                  return Ordering["greater"];
                                } else {
                                  return __lumo_match_error(__match_227);
                                }
                              } else {
                                return __lumo_match_error(__match_226);
                              }
                            })(122);
                            if ((__match_225[LUMO_TAG] === "less")) {
                              return __k_203(Bool["true"]);
                            } else if ((__match_225[LUMO_TAG] === "equal")) {
                              return __k_203(Bool["true"]);
                            } else {
                              return ((__match_225[LUMO_TAG] === "greater") ? __k_203(Bool["false"]) : __lumo_match_error(__match_225));
                            }
                          } else if ((__match_224[LUMO_TAG] === "false")) {
                            return __k_203(Bool["false"]);
                          } else {
                            return __lumo_match_error(__match_224);
                          }
                        });
                      });
                    } else if ((__match_214[LUMO_TAG] === "false")) {
                      return to_upper_char(__caps, c, (__lto_other_73) => {
                        let __cps_v_251;
                        const a = acc;
                        const b = __lto_other_73;
                        __cps_v_251 = (a + b);
                        return to_screaming_snake_loop(__caps, name, ((__lto_self_68) => {
                          const __lto_other_69 = 1;
                          const a = __lto_self_68;
                          const b = __lto_other_69;
                          return (a + b);
                        })(i), __cps_v_251, __k);
                      });
                    } else {
                      return __lumo_match_error(__match_214);
                    }
                  } else if ((is_upper[LUMO_TAG] === "false")) {
                    return to_upper_char(__caps, c, (__lto_other_81) => {
                      let __cps_v_250;
                      const a = acc;
                      const b = __lto_other_81;
                      __cps_v_250 = (a + b);
                      return to_screaming_snake_loop(__caps, name, ((__lto_self_76) => {
                        const __lto_other_77 = 1;
                        const a = __lto_self_76;
                        const b = __lto_other_77;
                        return (a + b);
                      })(i), __cps_v_250, __k);
                    });
                  } else {
                    return __lumo_match_error(is_upper);
                  }
                };
                let __match_234;
                let __match_240;
                __match_240 = ((__lto_other_5) => {
                  let __match_238;
                  const a = code;
                  const b = __lto_other_5;
                  __match_238 = ((a < b) ? Bool["true"] : Bool["false"]);
                  if ((__match_238[LUMO_TAG] === "true")) {
                    return Ordering["less"];
                  } else if ((__match_238[LUMO_TAG] === "false")) {
                    let __match_239;
                    const a = code;
                    const b = __lto_other_5;
                    if ((a === b)) {
                      __match_239 = Bool["true"];
                    } else {
                      __match_239 = Bool["false"];
                    }
                    if ((__match_239[LUMO_TAG] === "true")) {
                      return Ordering["equal"];
                    } else if ((__match_239[LUMO_TAG] === "false")) {
                      return Ordering["greater"];
                    } else {
                      return __lumo_match_error(__match_239);
                    }
                  } else {
                    return __lumo_match_error(__match_238);
                  }
                })(65);
                __match_234 = ((__match_240[LUMO_TAG] === "less") ? Bool["false"] : ((__match_240[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_240[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_240))));
                if ((__match_234[LUMO_TAG] === "true")) {
                  let __match_235;
                  __match_235 = ((__lto_other_9) => {
                    let __match_236;
                    const a = code;
                    const b = __lto_other_9;
                    __match_236 = ((a < b) ? Bool["true"] : Bool["false"]);
                    if ((__match_236[LUMO_TAG] === "true")) {
                      return Ordering["less"];
                    } else if ((__match_236[LUMO_TAG] === "false")) {
                      let __match_237;
                      const a = code;
                      const b = __lto_other_9;
                      if ((a === b)) {
                        __match_237 = Bool["true"];
                      } else {
                        __match_237 = Bool["false"];
                      }
                      if ((__match_237[LUMO_TAG] === "true")) {
                        return Ordering["equal"];
                      } else if ((__match_237[LUMO_TAG] === "false")) {
                        return Ordering["greater"];
                      } else {
                        return __lumo_match_error(__match_237);
                      }
                    } else {
                      return __lumo_match_error(__match_236);
                    }
                  })(90);
                  if ((__match_235[LUMO_TAG] === "less")) {
                    return __k_205(Bool["true"]);
                  } else if ((__match_235[LUMO_TAG] === "equal")) {
                    return __k_205(Bool["true"]);
                  } else {
                    return ((__match_235[LUMO_TAG] === "greater") ? __k_205(Bool["false"]) : __lumo_match_error(__match_235));
                  }
                } else if ((__match_234[LUMO_TAG] === "false")) {
                  return __k_205(Bool["false"]);
                } else {
                  return __lumo_match_error(__match_234);
                }
              });
            });
          } else {
            return __lumo_match_error(__cps_v_256);
          }
        };
        if ((__cps_v_257[LUMO_TAG] === "less")) {
          return __k_207(Bool["false"]);
        } else if ((__cps_v_257[LUMO_TAG] === "equal")) {
          return __k_207(Bool["true"]);
        } else {
          return ((__cps_v_257[LUMO_TAG] === "greater") ? __k_207(Bool["true"]) : __lumo_match_error(__cps_v_257));
        }
      };
      let __match_242;
      const a = i;
      const b = __lto_other_1;
      __match_242 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_242[LUMO_TAG] === "true")) {
        return __k_208(Ordering["less"]);
      } else if ((__match_242[LUMO_TAG] === "false")) {
        let __match_243;
        const a = i;
        const b = __lto_other_1;
        __match_243 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__match_243[LUMO_TAG] === "true")) {
          return __k_208(Ordering["equal"]);
        } else if ((__match_243[LUMO_TAG] === "false")) {
          return __k_208(Ordering["greater"]);
        } else {
          return __lumo_match_error(__match_243);
        }
      } else {
        return __lumo_match_error(__match_242);
      }
    });
  });
}

export function to_upper_char__lto(c) {
  const code = String.char_code_at(c, 0);
  let __match_247;
  let __match_246;
  __match_246 = ((__lto_other_85) => {
    let __match_244;
    const a = code;
    const b = __lto_other_85;
    __match_244 = ((a < b) ? Bool["true"] : Bool["false"]);
    if ((__match_244[LUMO_TAG] === "true")) {
      return Ordering["less"];
    } else if ((__match_244[LUMO_TAG] === "false")) {
      let __match_245;
      const a = code;
      const b = __lto_other_85;
      if ((a === b)) {
        __match_245 = Bool["true"];
      } else {
        __match_245 = Bool["false"];
      }
      if ((__match_245[LUMO_TAG] === "true")) {
        return Ordering["equal"];
      } else if ((__match_245[LUMO_TAG] === "false")) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_245);
      }
    } else {
      return __lumo_match_error(__match_244);
    }
  })(97);
  __match_247 = ((__match_246[LUMO_TAG] === "less") ? Bool["false"] : ((__match_246[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_246[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_246))));
  if ((__match_247[LUMO_TAG] === "true")) {
    let __match_251;
    let __match_250;
    const __lto_other_89 = 122;
    let __match_248;
    const a = code;
    const b = __lto_other_89;
    __match_248 = ((a < b) ? Bool["true"] : Bool["false"]);
    if ((__match_248[LUMO_TAG] === "true")) {
      __match_250 = Ordering["less"];
    } else if ((__match_248[LUMO_TAG] === "false")) {
      __match_250 = ((__match_249) => {
        if ((__match_249[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_249[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_249);
        }
      })(((a, b) => {
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })(code, __lto_other_89));
    } else {
      __match_250 = __lumo_match_error(__match_248);
    }
    if ((__match_250[LUMO_TAG] === "less")) {
      __match_251 = Bool["true"];
    } else if ((__match_250[LUMO_TAG] === "equal")) {
      __match_251 = Bool["true"];
    } else {
      __match_251 = ((__match_250[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_250));
    }
    if ((__match_251[LUMO_TAG] === "true")) {
      let __lto_code_92;
      const __lto_other_94 = 32;
      const a = code;
      const b = __lto_other_94;
      __lto_code_92 = (a - b);
      return fromCharCode(__lto_code_92);
    } else if ((__match_251[LUMO_TAG] === "false")) {
      return c;
    } else {
      return __lumo_match_error(__match_251);
    }
  } else if ((__match_247[LUMO_TAG] === "false")) {
    return c;
  } else {
    return __lumo_match_error(__match_247);
  }
}

export function keyword_variant__lto(__caps, kw, __k) {
  return to_upper_string(__caps, kw, (__lto_self_97) => {
    const __lto_other_98 = "_KW";
    return __k(((a, b) => {
      return (a + b);
    })(__lto_self_97, __lto_other_98));
  });
}

export function to_upper_string_loop__lto(__caps, s, i, acc, __k) {
  return __thunk(() => {
    return String.len(__caps, s, (__lto_other_102) => {
      const __k_212 = (__cps_v_261) => {
        const __k_211 = (__cps_v_260) => {
          if ((__cps_v_260[LUMO_TAG] === "true")) {
            return __k(acc);
          } else if ((__cps_v_260[LUMO_TAG] === "false")) {
            return String.char_at(__caps, s, i, (__cps_v_259) => {
              return to_upper_char(__caps, __cps_v_259, (__lto_other_110) => {
                let __cps_v_258;
                const a = acc;
                const b = __lto_other_110;
                __cps_v_258 = (a + b);
                return to_upper_string_loop(__caps, s, ((__lto_self_105) => {
                  const __lto_other_106 = 1;
                  const a = __lto_self_105;
                  const b = __lto_other_106;
                  return (a + b);
                })(i), __cps_v_258, __k);
              });
            });
          } else {
            return __lumo_match_error(__cps_v_260);
          }
        };
        if ((__cps_v_261[LUMO_TAG] === "less")) {
          return __k_211(Bool["false"]);
        } else if ((__cps_v_261[LUMO_TAG] === "equal")) {
          return __k_211(Bool["true"]);
        } else {
          return ((__cps_v_261[LUMO_TAG] === "greater") ? __k_211(Bool["true"]) : __lumo_match_error(__cps_v_261));
        }
      };
      let __match_254;
      const a = i;
      const b = __lto_other_102;
      __match_254 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_254[LUMO_TAG] === "true")) {
        return __k_212(Ordering["less"]);
      } else if ((__match_254[LUMO_TAG] === "false")) {
        let __match_255;
        const a = i;
        const b = __lto_other_102;
        __match_255 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__match_255[LUMO_TAG] === "true")) {
          return __k_212(Ordering["equal"]);
        } else if ((__match_255[LUMO_TAG] === "false")) {
          return __k_212(Ordering["greater"]);
        } else {
          return __lumo_match_error(__match_255);
        }
      } else {
        return __lumo_match_error(__match_254);
      }
    });
  });
}

export function symbol_variant__lto(sym) {
  let __match_256;
  __match_256 = ((__lto_other_114) => {
    const a = sym;
    const b = __lto_other_114;
    if ((a === b)) {
      return Bool["true"];
    } else {
      return Bool["false"];
    }
  })("#");
  if ((__match_256[LUMO_TAG] === "true")) {
    return "HASH";
  } else if ((__match_256[LUMO_TAG] === "false")) {
    let __match_257;
    const __lto_other_118 = "(";
    const a = sym;
    const b = __lto_other_118;
    if ((a === b)) {
      __match_257 = Bool["true"];
    } else {
      __match_257 = Bool["false"];
    }
    if ((__match_257[LUMO_TAG] === "true")) {
      return "L_PAREN";
    } else if ((__match_257[LUMO_TAG] === "false")) {
      let __match_258;
      const __lto_other_122 = ")";
      const a = sym;
      const b = __lto_other_122;
      if ((a === b)) {
        __match_258 = Bool["true"];
      } else {
        __match_258 = Bool["false"];
      }
      if ((__match_258[LUMO_TAG] === "true")) {
        return "R_PAREN";
      } else if ((__match_258[LUMO_TAG] === "false")) {
        let __match_259;
        const __lto_other_126 = "[";
        const a = sym;
        const b = __lto_other_126;
        if ((a === b)) {
          __match_259 = Bool["true"];
        } else {
          __match_259 = Bool["false"];
        }
        if ((__match_259[LUMO_TAG] === "true")) {
          return "L_BRACKET";
        } else if ((__match_259[LUMO_TAG] === "false")) {
          let __match_260;
          const __lto_other_130 = "]";
          const a = sym;
          const b = __lto_other_130;
          if ((a === b)) {
            __match_260 = Bool["true"];
          } else {
            __match_260 = Bool["false"];
          }
          if ((__match_260[LUMO_TAG] === "true")) {
            return "R_BRACKET";
          } else if ((__match_260[LUMO_TAG] === "false")) {
            let __match_261;
            const __lto_other_134 = "{";
            const a = sym;
            const b = __lto_other_134;
            if ((a === b)) {
              __match_261 = Bool["true"];
            } else {
              __match_261 = Bool["false"];
            }
            if ((__match_261[LUMO_TAG] === "true")) {
              return "L_BRACE";
            } else if ((__match_261[LUMO_TAG] === "false")) {
              let __match_262;
              const __lto_other_138 = "}";
              const a = sym;
              const b = __lto_other_138;
              if ((a === b)) {
                __match_262 = Bool["true"];
              } else {
                __match_262 = Bool["false"];
              }
              if ((__match_262[LUMO_TAG] === "true")) {
                return "R_BRACE";
              } else if ((__match_262[LUMO_TAG] === "false")) {
                let __match_263;
                const __lto_other_142 = ";";
                const a = sym;
                const b = __lto_other_142;
                if ((a === b)) {
                  __match_263 = Bool["true"];
                } else {
                  __match_263 = Bool["false"];
                }
                if ((__match_263[LUMO_TAG] === "true")) {
                  return "SEMICOLON";
                } else if ((__match_263[LUMO_TAG] === "false")) {
                  let __match_264;
                  const __lto_other_146 = ":";
                  const a = sym;
                  const b = __lto_other_146;
                  if ((a === b)) {
                    __match_264 = Bool["true"];
                  } else {
                    __match_264 = Bool["false"];
                  }
                  if ((__match_264[LUMO_TAG] === "true")) {
                    return "COLON";
                  } else if ((__match_264[LUMO_TAG] === "false")) {
                    let __match_265;
                    const __lto_other_150 = ",";
                    const a = sym;
                    const b = __lto_other_150;
                    if ((a === b)) {
                      __match_265 = Bool["true"];
                    } else {
                      __match_265 = Bool["false"];
                    }
                    if ((__match_265[LUMO_TAG] === "true")) {
                      return "COMMA";
                    } else if ((__match_265[LUMO_TAG] === "false")) {
                      let __match_266;
                      const __lto_other_154 = "=";
                      const a = sym;
                      const b = __lto_other_154;
                      if ((a === b)) {
                        __match_266 = Bool["true"];
                      } else {
                        __match_266 = Bool["false"];
                      }
                      if ((__match_266[LUMO_TAG] === "true")) {
                        return "EQUALS";
                      } else if ((__match_266[LUMO_TAG] === "false")) {
                        let __match_267;
                        const __lto_other_158 = ":=";
                        const a = sym;
                        const b = __lto_other_158;
                        if ((a === b)) {
                          __match_267 = Bool["true"];
                        } else {
                          __match_267 = Bool["false"];
                        }
                        if ((__match_267[LUMO_TAG] === "true")) {
                          return "COLON_EQ";
                        } else if ((__match_267[LUMO_TAG] === "false")) {
                          let __match_268;
                          const __lto_other_162 = "=>";
                          const a = sym;
                          const b = __lto_other_162;
                          if ((a === b)) {
                            __match_268 = Bool["true"];
                          } else {
                            __match_268 = Bool["false"];
                          }
                          if ((__match_268[LUMO_TAG] === "true")) {
                            return "FAT_ARROW";
                          } else if ((__match_268[LUMO_TAG] === "false")) {
                            let __match_269;
                            const __lto_other_166 = "->";
                            const a = sym;
                            const b = __lto_other_166;
                            if ((a === b)) {
                              __match_269 = Bool["true"];
                            } else {
                              __match_269 = Bool["false"];
                            }
                            if ((__match_269[LUMO_TAG] === "true")) {
                              return "ARROW";
                            } else if ((__match_269[LUMO_TAG] === "false")) {
                              let __match_270;
                              const __lto_other_170 = ".";
                              const a = sym;
                              const b = __lto_other_170;
                              if ((a === b)) {
                                __match_270 = Bool["true"];
                              } else {
                                __match_270 = Bool["false"];
                              }
                              if ((__match_270[LUMO_TAG] === "true")) {
                                return "DOT";
                              } else if ((__match_270[LUMO_TAG] === "false")) {
                                let __match_271;
                                const __lto_other_174 = "+";
                                const a = sym;
                                const b = __lto_other_174;
                                if ((a === b)) {
                                  __match_271 = Bool["true"];
                                } else {
                                  __match_271 = Bool["false"];
                                }
                                if ((__match_271[LUMO_TAG] === "true")) {
                                  return "PLUS";
                                } else if ((__match_271[LUMO_TAG] === "false")) {
                                  let __match_272;
                                  const __lto_other_178 = "-";
                                  const a = sym;
                                  const b = __lto_other_178;
                                  if ((a === b)) {
                                    __match_272 = Bool["true"];
                                  } else {
                                    __match_272 = Bool["false"];
                                  }
                                  if ((__match_272[LUMO_TAG] === "true")) {
                                    return "MINUS";
                                  } else if ((__match_272[LUMO_TAG] === "false")) {
                                    let __match_273;
                                    const __lto_other_182 = "*";
                                    const a = sym;
                                    const b = __lto_other_182;
                                    if ((a === b)) {
                                      __match_273 = Bool["true"];
                                    } else {
                                      __match_273 = Bool["false"];
                                    }
                                    if ((__match_273[LUMO_TAG] === "true")) {
                                      return "STAR";
                                    } else if ((__match_273[LUMO_TAG] === "false")) {
                                      let __match_274;
                                      const __lto_other_186 = "/";
                                      const a = sym;
                                      const b = __lto_other_186;
                                      if ((a === b)) {
                                        __match_274 = Bool["true"];
                                      } else {
                                        __match_274 = Bool["false"];
                                      }
                                      if ((__match_274[LUMO_TAG] === "true")) {
                                        return "SLASH";
                                      } else if ((__match_274[LUMO_TAG] === "false")) {
                                        let __match_275;
                                        const __lto_other_190 = "%";
                                        const a = sym;
                                        const b = __lto_other_190;
                                        if ((a === b)) {
                                          __match_275 = Bool["true"];
                                        } else {
                                          __match_275 = Bool["false"];
                                        }
                                        if ((__match_275[LUMO_TAG] === "true")) {
                                          return "PERCENT";
                                        } else if ((__match_275[LUMO_TAG] === "false")) {
                                          let __match_276;
                                          const __lto_other_194 = "!";
                                          const a = sym;
                                          const b = __lto_other_194;
                                          if ((a === b)) {
                                            __match_276 = Bool["true"];
                                          } else {
                                            __match_276 = Bool["false"];
                                          }
                                          if ((__match_276[LUMO_TAG] === "true")) {
                                            return "BANG";
                                          } else if ((__match_276[LUMO_TAG] === "false")) {
                                            let __match_277;
                                            const __lto_other_198 = "<";
                                            const a = sym;
                                            const b = __lto_other_198;
                                            if ((a === b)) {
                                              __match_277 = Bool["true"];
                                            } else {
                                              __match_277 = Bool["false"];
                                            }
                                            if ((__match_277[LUMO_TAG] === "true")) {
                                              return "LT";
                                            } else if ((__match_277[LUMO_TAG] === "false")) {
                                              let __match_278;
                                              const __lto_other_202 = ">";
                                              const a = sym;
                                              const b = __lto_other_202;
                                              if ((a === b)) {
                                                __match_278 = Bool["true"];
                                              } else {
                                                __match_278 = Bool["false"];
                                              }
                                              if ((__match_278[LUMO_TAG] === "true")) {
                                                return "GT";
                                              } else if ((__match_278[LUMO_TAG] === "false")) {
                                                let __match_279;
                                                const __lto_other_206 = "<=";
                                                const a = sym;
                                                const b = __lto_other_206;
                                                if ((a === b)) {
                                                  __match_279 = Bool["true"];
                                                } else {
                                                  __match_279 = Bool["false"];
                                                }
                                                if ((__match_279[LUMO_TAG] === "true")) {
                                                  return "LT_EQ";
                                                } else if ((__match_279[LUMO_TAG] === "false")) {
                                                  let __match_280;
                                                  const __lto_other_210 = ">=";
                                                  const a = sym;
                                                  const b = __lto_other_210;
                                                  if ((a === b)) {
                                                    __match_280 = Bool["true"];
                                                  } else {
                                                    __match_280 = Bool["false"];
                                                  }
                                                  if ((__match_280[LUMO_TAG] === "true")) {
                                                    return "GT_EQ";
                                                  } else if ((__match_280[LUMO_TAG] === "false")) {
                                                    let __match_281;
                                                    const __lto_other_214 = "==";
                                                    const a = sym;
                                                    const b = __lto_other_214;
                                                    if ((a === b)) {
                                                      __match_281 = Bool["true"];
                                                    } else {
                                                      __match_281 = Bool["false"];
                                                    }
                                                    if ((__match_281[LUMO_TAG] === "true")) {
                                                      return "EQ_EQ";
                                                    } else if ((__match_281[LUMO_TAG] === "false")) {
                                                      let __match_282;
                                                      const __lto_other_218 = "!=";
                                                      const a = sym;
                                                      const b = __lto_other_218;
                                                      if ((a === b)) {
                                                        __match_282 = Bool["true"];
                                                      } else {
                                                        __match_282 = Bool["false"];
                                                      }
                                                      if ((__match_282[LUMO_TAG] === "true")) {
                                                        return "BANG_EQ";
                                                      } else if ((__match_282[LUMO_TAG] === "false")) {
                                                        let __match_283;
                                                        const __lto_other_222 = "&&";
                                                        const a = sym;
                                                        const b = __lto_other_222;
                                                        if ((a === b)) {
                                                          __match_283 = Bool["true"];
                                                        } else {
                                                          __match_283 = Bool["false"];
                                                        }
                                                        if ((__match_283[LUMO_TAG] === "true")) {
                                                          return "AMP_AMP";
                                                        } else if ((__match_283[LUMO_TAG] === "false")) {
                                                          let __match_284;
                                                          const __lto_other_226 = "||";
                                                          const a = sym;
                                                          const b = __lto_other_226;
                                                          if ((a === b)) {
                                                            __match_284 = Bool["true"];
                                                          } else {
                                                            __match_284 = Bool["false"];
                                                          }
                                                          if ((__match_284[LUMO_TAG] === "true")) {
                                                            return "PIPE_PIPE";
                                                          } else if ((__match_284[LUMO_TAG] === "false")) {
                                                            let __match_285;
                                                            const __lto_other_230 = "_";
                                                            const a = sym;
                                                            const b = __lto_other_230;
                                                            if ((a === b)) {
                                                              __match_285 = Bool["true"];
                                                            } else {
                                                              __match_285 = Bool["false"];
                                                            }
                                                            if ((__match_285[LUMO_TAG] === "true")) {
                                                              return "UNDERSCORE";
                                                            } else if ((__match_285[LUMO_TAG] === "false")) {
                                                              const __lto_self_233 = "SYM_";
                                                              const a = __lto_self_233;
                                                              const b = sym;
                                                              return (a + b);
                                                            } else {
                                                              return __lumo_match_error(__match_285);
                                                            }
                                                          } else {
                                                            return __lumo_match_error(__match_284);
                                                          }
                                                        } else {
                                                          return __lumo_match_error(__match_283);
                                                        }
                                                      } else {
                                                        return __lumo_match_error(__match_282);
                                                      }
                                                    } else {
                                                      return __lumo_match_error(__match_281);
                                                    }
                                                  } else {
                                                    return __lumo_match_error(__match_280);
                                                  }
                                                } else {
                                                  return __lumo_match_error(__match_279);
                                                }
                                              } else {
                                                return __lumo_match_error(__match_278);
                                              }
                                            } else {
                                              return __lumo_match_error(__match_277);
                                            }
                                          } else {
                                            return __lumo_match_error(__match_276);
                                          }
                                        } else {
                                          return __lumo_match_error(__match_275);
                                        }
                                      } else {
                                        return __lumo_match_error(__match_274);
                                      }
                                    } else {
                                      return __lumo_match_error(__match_273);
                                    }
                                  } else {
                                    return __lumo_match_error(__match_272);
                                  }
                                } else {
                                  return __lumo_match_error(__match_271);
                                }
                              } else {
                                return __lumo_match_error(__match_270);
                              }
                            } else {
                              return __lumo_match_error(__match_269);
                            }
                          } else {
                            return __lumo_match_error(__match_268);
                          }
                        } else {
                          return __lumo_match_error(__match_267);
                        }
                      } else {
                        return __lumo_match_error(__match_266);
                      }
                    } else {
                      return __lumo_match_error(__match_265);
                    }
                  } else {
                    return __lumo_match_error(__match_264);
                  }
                } else {
                  return __lumo_match_error(__match_263);
                }
              } else {
                return __lumo_match_error(__match_262);
              }
            } else {
              return __lumo_match_error(__match_261);
            }
          } else {
            return __lumo_match_error(__match_260);
          }
        } else {
          return __lumo_match_error(__match_259);
        }
      } else {
        return __lumo_match_error(__match_258);
      }
    } else {
      return __lumo_match_error(__match_257);
    }
  } else {
    return __lumo_match_error(__match_256);
  }
}

export function collect_tokens_from_alts__lto(__caps, alts, kws, syms, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(StringPair["mk"](kws, syms));
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return String.char_code_at(__caps, name, 0, (code) => {
          let __match_288;
          let __match_295;
          __match_295 = ((__lto_other_238) => {
            let __match_293;
            const a = code;
            const b = __lto_other_238;
            __match_293 = ((a < b) ? Bool["true"] : Bool["false"]);
            if ((__match_293[LUMO_TAG] === "true")) {
              return Ordering["less"];
            } else if ((__match_293[LUMO_TAG] === "false")) {
              let __match_294;
              const a = code;
              const b = __lto_other_238;
              if ((a === b)) {
                __match_294 = Bool["true"];
              } else {
                __match_294 = Bool["false"];
              }
              if ((__match_294[LUMO_TAG] === "true")) {
                return Ordering["equal"];
              } else if ((__match_294[LUMO_TAG] === "false")) {
                return Ordering["greater"];
              } else {
                return __lumo_match_error(__match_294);
              }
            } else {
              return __lumo_match_error(__match_293);
            }
          })(65);
          __match_288 = ((__match_295[LUMO_TAG] === "less") ? Bool["false"] : ((__match_295[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_295[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_295))));
          if ((__match_288[LUMO_TAG] === "true")) {
            let __match_289;
            let __match_292;
            __match_292 = ((__lto_other_242) => {
              let __match_290;
              const a = code;
              const b = __lto_other_242;
              __match_290 = ((a < b) ? Bool["true"] : Bool["false"]);
              if ((__match_290[LUMO_TAG] === "true")) {
                return Ordering["less"];
              } else if ((__match_290[LUMO_TAG] === "false")) {
                let __match_291;
                const a = code;
                const b = __lto_other_242;
                if ((a === b)) {
                  __match_291 = Bool["true"];
                } else {
                  __match_291 = Bool["false"];
                }
                if ((__match_291[LUMO_TAG] === "true")) {
                  return Ordering["equal"];
                } else if ((__match_291[LUMO_TAG] === "false")) {
                  return Ordering["greater"];
                } else {
                  return __lumo_match_error(__match_291);
                }
              } else {
                return __lumo_match_error(__match_290);
              }
            })(90);
            __match_289 = ((__match_292[LUMO_TAG] === "less") ? Bool["true"] : ((__match_292[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_292[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_292))));
            if ((__match_289[LUMO_TAG] === "true")) {
              return collect_tokens_from_alts(__caps, rest, kws, syms, __k);
            } else if ((__match_289[LUMO_TAG] === "false")) {
              return collect_alt_token(__caps, name, rest, kws, syms, __k);
            } else {
              return __lumo_match_error(__match_289);
            }
          } else if ((__match_288[LUMO_TAG] === "false")) {
            return collect_alt_token(__caps, name, rest, kws, syms, __k);
          } else {
            return __lumo_match_error(__match_288);
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

export function string_lt_loop__lto(__caps, a, b, i, __k) {
  return __thunk(() => {
    return String.len(__caps, a, (__lto_other_246) => {
      const __k_230 = (__cps_v_267) => {
        const __k_229 = (__cps_v_266) => {
          if ((__cps_v_266[LUMO_TAG] === "true")) {
            return String.len(__caps, b, (__lto_other_250) => {
              const __k_227 = (__cps_v_265) => {
                const __k_226 = (__cps_v_264) => {
                  if ((__cps_v_264[LUMO_TAG] === "true")) {
                    return __k(Bool["false"]);
                  } else if ((__cps_v_264[LUMO_TAG] === "false")) {
                    return __k(Bool["true"]);
                  } else {
                    return __lumo_match_error(__cps_v_264);
                  }
                };
                if ((__cps_v_265[LUMO_TAG] === "less")) {
                  return __k_226(Bool["false"]);
                } else if ((__cps_v_265[LUMO_TAG] === "equal")) {
                  return __k_226(Bool["true"]);
                } else {
                  return ((__cps_v_265[LUMO_TAG] === "greater") ? __k_226(Bool["true"]) : __lumo_match_error(__cps_v_265));
                }
              };
              let __match_311;
              const a = i;
              const b = __lto_other_250;
              __match_311 = ((a < b) ? Bool["true"] : Bool["false"]);
              if ((__match_311[LUMO_TAG] === "true")) {
                return __k_227(Ordering["less"]);
              } else if ((__match_311[LUMO_TAG] === "false")) {
                let __match_312;
                const a = i;
                const b = __lto_other_250;
                __match_312 = ((a === b) ? Bool["true"] : Bool["false"]);
                if ((__match_312[LUMO_TAG] === "true")) {
                  return __k_227(Ordering["equal"]);
                } else if ((__match_312[LUMO_TAG] === "false")) {
                  return __k_227(Ordering["greater"]);
                } else {
                  return __lumo_match_error(__match_312);
                }
              } else {
                return __lumo_match_error(__match_311);
              }
            });
          } else if ((__cps_v_266[LUMO_TAG] === "false")) {
            return String.len(__caps, b, (__lto_other_254) => {
              const __k_223 = (__cps_v_263) => {
                const __k_222 = (__cps_v_262) => {
                  if ((__cps_v_262[LUMO_TAG] === "true")) {
                    return __k(Bool["false"]);
                  } else if ((__cps_v_262[LUMO_TAG] === "false")) {
                    return String.char_code_at(__caps, a, i, (ca) => {
                      return String.char_code_at(__caps, b, i, (cb) => {
                        let __match_298;
                        let __match_305;
                        __match_305 = ((__lto_other_258) => {
                          let __match_303;
                          const a = ca;
                          const b = __lto_other_258;
                          __match_303 = ((a < b) ? Bool["true"] : Bool["false"]);
                          if ((__match_303[LUMO_TAG] === "true")) {
                            return Ordering["less"];
                          } else if ((__match_303[LUMO_TAG] === "false")) {
                            let __match_304;
                            const a = ca;
                            const b = __lto_other_258;
                            if ((a === b)) {
                              __match_304 = Bool["true"];
                            } else {
                              __match_304 = Bool["false"];
                            }
                            if ((__match_304[LUMO_TAG] === "true")) {
                              return Ordering["equal"];
                            } else if ((__match_304[LUMO_TAG] === "false")) {
                              return Ordering["greater"];
                            } else {
                              return __lumo_match_error(__match_304);
                            }
                          } else {
                            return __lumo_match_error(__match_303);
                          }
                        })(cb);
                        __match_298 = ((__match_305[LUMO_TAG] === "less") ? Bool["true"] : ((__match_305[LUMO_TAG] === "equal") ? Bool["false"] : ((__match_305[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_305))));
                        if ((__match_298[LUMO_TAG] === "true")) {
                          return __k(Bool["true"]);
                        } else if ((__match_298[LUMO_TAG] === "false")) {
                          let __match_299;
                          let __match_302;
                          __match_302 = ((__lto_other_262) => {
                            let __match_300;
                            const a = cb;
                            const b = __lto_other_262;
                            __match_300 = ((a < b) ? Bool["true"] : Bool["false"]);
                            if ((__match_300[LUMO_TAG] === "true")) {
                              return Ordering["less"];
                            } else if ((__match_300[LUMO_TAG] === "false")) {
                              let __match_301;
                              const a = cb;
                              const b = __lto_other_262;
                              if ((a === b)) {
                                __match_301 = Bool["true"];
                              } else {
                                __match_301 = Bool["false"];
                              }
                              if ((__match_301[LUMO_TAG] === "true")) {
                                return Ordering["equal"];
                              } else if ((__match_301[LUMO_TAG] === "false")) {
                                return Ordering["greater"];
                              } else {
                                return __lumo_match_error(__match_301);
                              }
                            } else {
                              return __lumo_match_error(__match_300);
                            }
                          })(ca);
                          __match_299 = ((__match_302[LUMO_TAG] === "less") ? Bool["true"] : ((__match_302[LUMO_TAG] === "equal") ? Bool["false"] : ((__match_302[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_302))));
                          if ((__match_299[LUMO_TAG] === "true")) {
                            return __k(Bool["false"]);
                          } else if ((__match_299[LUMO_TAG] === "false")) {
                            return string_lt_loop(__caps, a, b, ((__lto_self_265) => {
                              const __lto_other_266 = 1;
                              const a = __lto_self_265;
                              const b = __lto_other_266;
                              return (a + b);
                            })(i), __k);
                          } else {
                            return __lumo_match_error(__match_299);
                          }
                        } else {
                          return __lumo_match_error(__match_298);
                        }
                      });
                    });
                  } else {
                    return __lumo_match_error(__cps_v_262);
                  }
                };
                if ((__cps_v_263[LUMO_TAG] === "less")) {
                  return __k_222(Bool["false"]);
                } else if ((__cps_v_263[LUMO_TAG] === "equal")) {
                  return __k_222(Bool["true"]);
                } else {
                  return ((__cps_v_263[LUMO_TAG] === "greater") ? __k_222(Bool["true"]) : __lumo_match_error(__cps_v_263));
                }
              };
              let __match_307;
              const a = i;
              const b = __lto_other_254;
              __match_307 = ((a < b) ? Bool["true"] : Bool["false"]);
              if ((__match_307[LUMO_TAG] === "true")) {
                return __k_223(Ordering["less"]);
              } else if ((__match_307[LUMO_TAG] === "false")) {
                let __match_308;
                const a = i;
                const b = __lto_other_254;
                __match_308 = ((a === b) ? Bool["true"] : Bool["false"]);
                if ((__match_308[LUMO_TAG] === "true")) {
                  return __k_223(Ordering["equal"]);
                } else if ((__match_308[LUMO_TAG] === "false")) {
                  return __k_223(Ordering["greater"]);
                } else {
                  return __lumo_match_error(__match_308);
                }
              } else {
                return __lumo_match_error(__match_307);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_266);
          }
        };
        if ((__cps_v_267[LUMO_TAG] === "less")) {
          return __k_229(Bool["false"]);
        } else if ((__cps_v_267[LUMO_TAG] === "equal")) {
          return __k_229(Bool["true"]);
        } else {
          return ((__cps_v_267[LUMO_TAG] === "greater") ? __k_229(Bool["true"]) : __lumo_match_error(__cps_v_267));
        }
      };
      let __match_314;
      const a = i;
      const b = __lto_other_246;
      __match_314 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_314[LUMO_TAG] === "true")) {
        return __k_230(Ordering["less"]);
      } else if ((__match_314[LUMO_TAG] === "false")) {
        let __match_315;
        const a = i;
        const b = __lto_other_246;
        __match_315 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__match_315[LUMO_TAG] === "true")) {
          return __k_230(Ordering["equal"]);
        } else if ((__match_315[LUMO_TAG] === "false")) {
          return __k_230(Ordering["greater"]);
        } else {
          return __lumo_match_error(__match_315);
        }
      } else {
        return __lumo_match_error(__match_314);
      }
    });
  });
}

export function is_token_only_alternatives__lto(__caps, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(Bool["true"]);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return String.char_code_at(__caps, name, 0, (code) => {
          const __k_235 = (is_upper) => {
            if ((is_upper[LUMO_TAG] === "true")) {
              return __k(Bool["false"]);
            } else if ((is_upper[LUMO_TAG] === "false")) {
              return is_token_only_alternatives(__caps, rest, __k);
            } else {
              return __lumo_match_error(is_upper);
            }
          };
          let __match_319;
          let __match_325;
          __match_325 = ((__lto_other_270) => {
            let __match_323;
            const a = code;
            const b = __lto_other_270;
            __match_323 = ((a < b) ? Bool["true"] : Bool["false"]);
            if ((__match_323[LUMO_TAG] === "true")) {
              return Ordering["less"];
            } else if ((__match_323[LUMO_TAG] === "false")) {
              let __match_324;
              const a = code;
              const b = __lto_other_270;
              if ((a === b)) {
                __match_324 = Bool["true"];
              } else {
                __match_324 = Bool["false"];
              }
              if ((__match_324[LUMO_TAG] === "true")) {
                return Ordering["equal"];
              } else if ((__match_324[LUMO_TAG] === "false")) {
                return Ordering["greater"];
              } else {
                return __lumo_match_error(__match_324);
              }
            } else {
              return __lumo_match_error(__match_323);
            }
          })(65);
          __match_319 = ((__match_325[LUMO_TAG] === "less") ? Bool["false"] : ((__match_325[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_325[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_325))));
          if ((__match_319[LUMO_TAG] === "true")) {
            let __match_320;
            __match_320 = ((__lto_other_274) => {
              let __match_321;
              const a = code;
              const b = __lto_other_274;
              __match_321 = ((a < b) ? Bool["true"] : Bool["false"]);
              if ((__match_321[LUMO_TAG] === "true")) {
                return Ordering["less"];
              } else if ((__match_321[LUMO_TAG] === "false")) {
                let __match_322;
                const a = code;
                const b = __lto_other_274;
                if ((a === b)) {
                  __match_322 = Bool["true"];
                } else {
                  __match_322 = Bool["false"];
                }
                if ((__match_322[LUMO_TAG] === "true")) {
                  return Ordering["equal"];
                } else if ((__match_322[LUMO_TAG] === "false")) {
                  return Ordering["greater"];
                } else {
                  return __lumo_match_error(__match_322);
                }
              } else {
                return __lumo_match_error(__match_321);
              }
            })(90);
            if ((__match_320[LUMO_TAG] === "less")) {
              return __k_235(Bool["true"]);
            } else if ((__match_320[LUMO_TAG] === "equal")) {
              return __k_235(Bool["true"]);
            } else {
              return ((__match_320[LUMO_TAG] === "greater") ? __k_235(Bool["false"]) : __lumo_match_error(__match_320));
            }
          } else if ((__match_319[LUMO_TAG] === "false")) {
            return __k_235(Bool["false"]);
          } else {
            return __lumo_match_error(__match_319);
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

export function emit_named_tokens__lto(__caps, s, tokens, __k) {
  return __thunk(() => {
    if ((tokens[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((tokens[LUMO_TAG] === "cons")) {
      const tok = tokens.args[0];
      const rest = tokens.args[1];
      const __lto_other_342 = "    ";
      let __lto_self_339;
      const a = s;
      const b = __lto_other_342;
      __lto_self_339 = (a + b);
      return to_screaming_snake(__caps, tok, (__lto_other_340) => {
        let __lto_self_337;
        const a = __lto_self_339;
        const b = __lto_other_340;
        __lto_self_337 = (a + b);
        const __lto_other_338 = ",\n";
        let __cps_v_268;
        const a_0 = __lto_self_337;
        const b_1 = __lto_other_338;
        __cps_v_268 = (a_0 + b_1);
        return emit_named_tokens(__caps, __cps_v_268, rest, __k);
      });
    } else {
      return __lumo_match_error(tokens);
    }
  });
}

export function emit_keywords__lto(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __lto_other_350 = "    // Keywords\n";
      let s2;
      const a = s;
      const b = __lto_other_350;
      s2 = (a + b);
      return emit_keywords_items(__caps, s2, kws, __k);
    }
  });
}

export function emit_keywords_items__lto(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      const __lto_self_359 = "    ";
      return keyword_variant(__caps, kw, (__lto_other_360) => {
        let __lto_self_357;
        const a = __lto_self_359;
        const b = __lto_other_360;
        __lto_self_357 = (a + b);
        const __lto_other_358 = ", // '";
        let __lto_self_355;
        const a_0 = __lto_self_357;
        const b_1 = __lto_other_358;
        __lto_self_355 = (a_0 + b_1);
        let __lto_self_353;
        const a_2 = __lto_self_355;
        const b_3 = kw;
        __lto_self_353 = (a_2 + b_3);
        const __lto_other_354 = "'\n";
        let line;
        const a_4 = __lto_self_353;
        const b_5 = __lto_other_354;
        line = (a_4 + b_5);
        return emit_keywords_items(__caps, ((__lto_self_369) => {
          const a = __lto_self_369;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_symbols__lto(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      const __k_241 = (s2) => {
        return emit_symbols_items(__caps, s2, syms, __k);
      };
      if ((rest[LUMO_TAG] === "nil")) {
        return __k_241(s);
      } else {
        const __lto_other_374 = "    // Symbols\n";
        return __k_241(((a, b) => {
          return (a + b);
        })(s, __lto_other_374));
      }
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function emit_symbols_items__lto(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      const __lto_self_383 = "    ";
      return symbol_variant(__caps, sym, (__lto_other_384) => {
        let __lto_self_381;
        const a = __lto_self_383;
        const b = __lto_other_384;
        __lto_self_381 = (a + b);
        const __lto_other_382 = ", // '";
        let __lto_self_379;
        const a_0 = __lto_self_381;
        const b_1 = __lto_other_382;
        __lto_self_379 = (a_0 + b_1);
        let __lto_self_377;
        const a_2 = __lto_self_379;
        const b_3 = sym;
        __lto_self_377 = (a_2 + b_3);
        const __lto_other_378 = "'\n";
        let line;
        const a_4 = __lto_self_377;
        const b_5 = __lto_other_378;
        line = (a_4 + b_5);
        return emit_symbols_items(__caps, ((__lto_self_393) => {
          const a = __lto_self_393;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function emit_node_kinds__lto(__caps, s, rules, __k) {
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
            let __lto_self_401;
            const a = __lto_self_403;
            const b = __lto_other_404;
            __lto_self_401 = (a + b);
            const __lto_other_402 = ", // ";
            let __lto_self_399;
            const a_0 = __lto_self_401;
            const b_1 = __lto_other_402;
            __lto_self_399 = (a_0 + b_1);
            let __lto_self_397;
            const a_2 = __lto_self_399;
            const b_3 = name;
            __lto_self_397 = (a_2 + b_3);
            const __lto_other_398 = "\n";
            let line;
            const a_4 = __lto_self_397;
            const b_5 = __lto_other_398;
            line = (a_4 + b_5);
            return emit_node_kinds(__caps, ((__lto_self_413) => {
              const a = __lto_self_413;
              const b = line;
              return (a + b);
            })(s), rest, __k);
          });
        } else if ((body[LUMO_TAG] === "alternatives")) {
          const alts = body.args[0];
          return is_token_only_alternatives(__caps, alts, (__cps_v_269) => {
            if ((__cps_v_269[LUMO_TAG] === "true")) {
              const __lto_self_423 = "    ";
              return to_screaming_snake(__caps, name, (__lto_other_424) => {
                let __lto_self_421;
                const a = __lto_self_423;
                const b = __lto_other_424;
                __lto_self_421 = (a + b);
                const __lto_other_422 = ", // ";
                let __lto_self_419;
                const a_0 = __lto_self_421;
                const b_1 = __lto_other_422;
                __lto_self_419 = (a_0 + b_1);
                let __lto_self_417;
                const a_2 = __lto_self_419;
                const b_3 = name;
                __lto_self_417 = (a_2 + b_3);
                const __lto_other_418 = " (token wrapper)\n";
                let line;
                const a_4 = __lto_self_417;
                const b_5 = __lto_other_418;
                line = (a_4 + b_5);
                return emit_node_kinds(__caps, ((__lto_self_433) => {
                  const a = __lto_self_433;
                  const b = line;
                  return (a + b);
                })(s), rest, __k);
              });
            } else if ((__cps_v_269[LUMO_TAG] === "false")) {
              return emit_node_kinds(__caps, s, rest, __k);
            } else {
              return __lumo_match_error(__cps_v_269);
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

export function emit_from_keyword__lto(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __lto_other_438 = "\n    pub fn from_keyword(text: &str) -> Option<Self> {\n";
      let s;
      const a = s;
      const b = __lto_other_438;
      s = (a + b);
      const __lto_other_442 = "        match text {\n";
      let s_0;
      const a_1 = s;
      const b_2 = __lto_other_442;
      s_0 = (a_1 + b_2);
      return emit_keyword_arms(__caps, s_0, kws, (s) => {
        const __lto_other_446 = "            _ => None,\n";
        let s_0;
        const a = s;
        const b = __lto_other_446;
        s_0 = (a + b);
        const __lto_other_450 = "        }\n";
        let s_1;
        const a_2 = s_0;
        const b_3 = __lto_other_450;
        s_1 = (a_2 + b_3);
        const __lto_other_454 = "    }\n";
        let s_4;
        const a_5 = s_1;
        const b_6 = __lto_other_454;
        s_4 = (a_5 + b_6);
        return __k(s_4);
      });
    }
  });
}

export function emit_keyword_arms__lto(__caps, s, kws, __k) {
  return __thunk(() => {
    if ((kws[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((kws[LUMO_TAG] === "cons")) {
      const kw = kws.args[0];
      const rest = kws.args[1];
      const __lto_self_463 = "            \"";
      let __lto_self_461;
      const a = __lto_self_463;
      const b = kw;
      __lto_self_461 = (a + b);
      const __lto_other_462 = "\" => Some(Self::";
      let __lto_self_459;
      const a_0 = __lto_self_461;
      const b_1 = __lto_other_462;
      __lto_self_459 = (a_0 + b_1);
      return keyword_variant(__caps, kw, (__lto_other_460) => {
        let __lto_self_457;
        const a = __lto_self_459;
        const b = __lto_other_460;
        __lto_self_457 = (a + b);
        const __lto_other_458 = "),\n";
        let line;
        const a_0 = __lto_self_457;
        const b_1 = __lto_other_458;
        line = (a_0 + b_1);
        return emit_keyword_arms(__caps, ((__lto_self_473) => {
          const a = __lto_self_473;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(kws);
    }
  });
}

export function emit_from_symbol__lto(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else {
      const __lto_other_478 = "\n    pub fn from_symbol(text: &str) -> Option<Self> {\n";
      let s;
      const a = s;
      const b = __lto_other_478;
      s = (a + b);
      const __lto_other_482 = "        match text {\n";
      let s_0;
      const a_1 = s;
      const b_2 = __lto_other_482;
      s_0 = (a_1 + b_2);
      return emit_symbol_arms(__caps, s_0, syms, (s) => {
        const __lto_other_486 = "            _ => None,\n";
        let s_0;
        const a = s;
        const b = __lto_other_486;
        s_0 = (a + b);
        const __lto_other_490 = "        }\n";
        let s_1;
        const a_2 = s_0;
        const b_3 = __lto_other_490;
        s_1 = (a_2 + b_3);
        const __lto_other_494 = "    }\n";
        let s_4;
        const a_5 = s_1;
        const b_6 = __lto_other_494;
        s_4 = (a_5 + b_6);
        return __k(s_4);
      });
    }
  });
}

export function emit_symbol_arms__lto(__caps, s, syms, __k) {
  return __thunk(() => {
    if ((syms[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((syms[LUMO_TAG] === "cons")) {
      const sym = syms.args[0];
      const rest = syms.args[1];
      const __lto_self_503 = "            \"";
      let __lto_self_501;
      const a = __lto_self_503;
      const b = sym;
      __lto_self_501 = (a + b);
      const __lto_other_502 = "\" => Some(Self::";
      let __lto_self_499;
      const a_0 = __lto_self_501;
      const b_1 = __lto_other_502;
      __lto_self_499 = (a_0 + b_1);
      return symbol_variant(__caps, sym, (__lto_other_500) => {
        let __lto_self_497;
        const a = __lto_self_499;
        const b = __lto_other_500;
        __lto_self_497 = (a + b);
        const __lto_other_498 = "),\n";
        let line;
        const a_0 = __lto_self_497;
        const b_1 = __lto_other_498;
        line = (a_0 + b_1);
        return emit_symbol_arms(__caps, ((__lto_self_513) => {
          const a = __lto_self_513;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      });
    } else {
      return __lumo_match_error(syms);
    }
  });
}

export function emit_struct_node__lto(__caps, s, name, elems, token_defs, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    const __lto_other_550 = "pub struct ";
    let __lto_self_547;
    const a = s;
    const b = __lto_other_550;
    __lto_self_547 = (a + b);
    let __lto_self_545;
    const a_0 = __lto_self_547;
    const b_1 = name;
    __lto_self_545 = (a_0 + b_1);
    const __lto_other_546 = "<'a>(pub(crate) &'a SyntaxNode);\n\n";
    let s;
    const a_2 = __lto_self_545;
    const b_3 = __lto_other_546;
    s = (a_2 + b_3);
    const __lto_other_562 = "impl<'a> AstNode<'a> for ";
    let __lto_self_559;
    const a_4 = s;
    const b_5 = __lto_other_562;
    __lto_self_559 = (a_4 + b_5);
    let __lto_self_557;
    const a_6 = __lto_self_559;
    const b_7 = name;
    __lto_self_557 = (a_6 + b_7);
    const __lto_other_558 = "<'a> {\n";
    let s_8;
    const a_9 = __lto_self_557;
    const b_10 = __lto_other_558;
    s_8 = (a_9 + b_10);
    const __lto_other_570 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
    let s_11;
    const a_12 = s_8;
    const b_13 = __lto_other_570;
    s_11 = (a_12 + b_13);
    const __lto_other_578 = "        (node.kind == SyntaxKind::";
    let __lto_self_575;
    const a_14 = s_11;
    const b_15 = __lto_other_578;
    __lto_self_575 = (a_14 + b_15);
    let __lto_self_573;
    const a_16 = __lto_self_575;
    const b_17 = kind;
    __lto_self_573 = (a_16 + b_17);
    const __lto_other_574 = ").then(|| Self(node))\n";
    let s_18;
    const a_19 = __lto_self_573;
    const b_20 = __lto_other_574;
    s_18 = (a_19 + b_20);
    const __lto_other_586 = "    }\n";
    let s_21;
    const a_22 = s_18;
    const b_23 = __lto_other_586;
    s_21 = (a_22 + b_23);
    const __lto_other_590 = "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n";
    let s_24;
    const a_25 = s_21;
    const b_26 = __lto_other_590;
    s_24 = (a_25 + b_26);
    const __lto_other_594 = "}\n\n";
    let s_27;
    const a_28 = s_24;
    const b_29 = __lto_other_594;
    s_27 = (a_28 + b_29);
    return emit_accessors(__caps, s_27, name, elems, token_defs, __k);
  });
}

export function emit_token_accessor__lto(__caps, s, label, t, repeated, __k) {
  return token_kind_from_ref(__caps, t, (kind) => {
    if ((repeated[LUMO_TAG] === "true")) {
      const __lto_other_618 = "    pub fn ";
      let __lto_self_615;
      const a = s;
      const b = __lto_other_618;
      __lto_self_615 = (a + b);
      let __lto_self_613;
      const a_0 = __lto_self_615;
      const b_1 = label;
      __lto_self_613 = (a_0 + b_1);
      const __lto_other_614 = "(&self) -> impl Iterator<Item = &'a LosslessToken> + 'a {\n";
      let s;
      const a_2 = __lto_self_613;
      const b_3 = __lto_other_614;
      s = (a_2 + b_3);
      const __lto_other_626 = "        self.0.children.iter().filter_map(|c| match c {\n";
      let s_4;
      const a_5 = s;
      const b_6 = __lto_other_626;
      s_4 = (a_5 + b_6);
      const __lto_other_634 = "            SyntaxElement::Token(t) if t.kind == SyntaxKind::";
      let __lto_self_631;
      const a_7 = s_4;
      const b_8 = __lto_other_634;
      __lto_self_631 = (a_7 + b_8);
      let __lto_self_629;
      const a_9 = __lto_self_631;
      const b_10 = kind;
      __lto_self_629 = (a_9 + b_10);
      const __lto_other_630 = " => Some(t),\n";
      let s_11;
      const a_12 = __lto_self_629;
      const b_13 = __lto_other_630;
      s_11 = (a_12 + b_13);
      const __lto_other_642 = "            _ => None,\n";
      let s_14;
      const a_15 = s_11;
      const b_16 = __lto_other_642;
      s_14 = (a_15 + b_16);
      const __lto_other_646 = "        })\n";
      let s_17;
      const a_18 = s_14;
      const b_19 = __lto_other_646;
      s_17 = (a_18 + b_19);
      const __lto_other_650 = "    }\n";
      return __k(((a, b) => {
        return (a + b);
      })(s_17, __lto_other_650));
    } else if ((repeated[LUMO_TAG] === "false")) {
      const __lto_other_658 = "    pub fn ";
      let __lto_self_655;
      const a = s;
      const b = __lto_other_658;
      __lto_self_655 = (a + b);
      let __lto_self_653;
      const a_0 = __lto_self_655;
      const b_1 = label;
      __lto_self_653 = (a_0 + b_1);
      const __lto_other_654 = "(&self) -> Option<&'a LosslessToken> {\n";
      let s;
      const a_2 = __lto_self_653;
      const b_3 = __lto_other_654;
      s = (a_2 + b_3);
      const __lto_other_666 = "        self.0.children.iter().find_map(|c| match c {\n";
      let s_4;
      const a_5 = s;
      const b_6 = __lto_other_666;
      s_4 = (a_5 + b_6);
      const __lto_other_674 = "            SyntaxElement::Token(t) if t.kind == SyntaxKind::";
      let __lto_self_671;
      const a_7 = s_4;
      const b_8 = __lto_other_674;
      __lto_self_671 = (a_7 + b_8);
      let __lto_self_669;
      const a_9 = __lto_self_671;
      const b_10 = kind;
      __lto_self_669 = (a_9 + b_10);
      const __lto_other_670 = " => Some(t),\n";
      let s_11;
      const a_12 = __lto_self_669;
      const b_13 = __lto_other_670;
      s_11 = (a_12 + b_13);
      const __lto_other_682 = "            _ => None,\n";
      let s_14;
      const a_15 = s_11;
      const b_16 = __lto_other_682;
      s_14 = (a_15 + b_16);
      const __lto_other_686 = "        })\n";
      let s_17;
      const a_18 = s_14;
      const b_19 = __lto_other_686;
      s_17 = (a_18 + b_19);
      const __lto_other_690 = "    }\n";
      return __k(((a, b) => {
        return (a + b);
      })(s_17, __lto_other_690));
    } else {
      return __lumo_match_error(repeated);
    }
  });
}

export function emit_node_accessor__lto(s, label, node_name, repeated) {
  if ((repeated[LUMO_TAG] === "true")) {
    return ((s) => {
      return ((s) => {
        return ((s) => {
          return ((s) => {
            return ((s) => {
              const __lto_other_738 = "    }\n";
              const a = s;
              const b = __lto_other_738;
              return (a + b);
            })(((__lto_self_733) => {
              const __lto_other_734 = "        })\n";
              const a = __lto_self_733;
              const b = __lto_other_734;
              return (a + b);
            })(s));
          })(((__lto_self_729) => {
            const __lto_other_730 = "            _ => None,\n";
            const a = __lto_self_729;
            const b = __lto_other_730;
            return (a + b);
          })(s));
        })(((__lto_self_717) => {
          const __lto_other_718 = "::cast(n),\n";
          const a = __lto_self_717;
          const b = __lto_other_718;
          return (a + b);
        })(((__lto_self_719) => {
          const a = __lto_self_719;
          const b = node_name;
          return (a + b);
        })(((__lto_self_721) => {
          const __lto_other_722 = "            SyntaxElement::Node(n) => ";
          const a = __lto_self_721;
          const b = __lto_other_722;
          return (a + b);
        })(s))));
      })(((__lto_self_713) => {
        const __lto_other_714 = "        self.0.children.iter().filter_map(|c| match c {\n";
        const a = __lto_self_713;
        const b = __lto_other_714;
        return (a + b);
      })(s));
    })(((__lto_self_693) => {
      const __lto_other_694 = "<'a>> + 'a {\n";
      const a = __lto_self_693;
      const b = __lto_other_694;
      return (a + b);
    })(((__lto_self_695) => {
      const a = __lto_self_695;
      const b = node_name;
      return (a + b);
    })(((__lto_self_697) => {
      const __lto_other_698 = "(&self) -> impl Iterator<Item = ";
      const a = __lto_self_697;
      const b = __lto_other_698;
      return (a + b);
    })(((__lto_self_699) => {
      const a = __lto_self_699;
      const b = label;
      return (a + b);
    })(((__lto_self_701) => {
      const __lto_other_702 = "    pub fn ";
      const a = __lto_self_701;
      const b = __lto_other_702;
      return (a + b);
    })(s))))));
  } else if ((repeated[LUMO_TAG] === "false")) {
    return ((s) => {
      return ((s) => {
        return ((s) => {
          return ((s) => {
            return ((s) => {
              const __lto_other_786 = "    }\n";
              const a = s;
              const b = __lto_other_786;
              return (a + b);
            })(((__lto_self_781) => {
              const __lto_other_782 = "        })\n";
              const a = __lto_self_781;
              const b = __lto_other_782;
              return (a + b);
            })(s));
          })(((__lto_self_777) => {
            const __lto_other_778 = "            _ => None,\n";
            const a = __lto_self_777;
            const b = __lto_other_778;
            return (a + b);
          })(s));
        })(((__lto_self_765) => {
          const __lto_other_766 = "::cast(n),\n";
          const a = __lto_self_765;
          const b = __lto_other_766;
          return (a + b);
        })(((__lto_self_767) => {
          const a = __lto_self_767;
          const b = node_name;
          return (a + b);
        })(((__lto_self_769) => {
          const __lto_other_770 = "            SyntaxElement::Node(n) => ";
          const a = __lto_self_769;
          const b = __lto_other_770;
          return (a + b);
        })(s))));
      })(((__lto_self_761) => {
        const __lto_other_762 = "        self.0.children.iter().find_map(|c| match c {\n";
        const a = __lto_self_761;
        const b = __lto_other_762;
        return (a + b);
      })(s));
    })(((__lto_self_741) => {
      const __lto_other_742 = "<'a>> {\n";
      const a = __lto_self_741;
      const b = __lto_other_742;
      return (a + b);
    })(((__lto_self_743) => {
      const a = __lto_self_743;
      const b = node_name;
      return (a + b);
    })(((__lto_self_745) => {
      const __lto_other_746 = "(&self) -> Option<";
      const a = __lto_self_745;
      const b = __lto_other_746;
      return (a + b);
    })(((__lto_self_747) => {
      const a = __lto_self_747;
      const b = label;
      return (a + b);
    })(((__lto_self_749) => {
      const __lto_other_750 = "    pub fn ";
      const a = __lto_self_749;
      const b = __lto_other_750;
      return (a + b);
    })(s))))));
  } else {
    return __lumo_match_error(repeated);
  }
}

export function emit_enum_node__lto(__caps, s, name, alts, __k) {
  return __thunk(() => {
    const __lto_other_794 = "pub enum ";
    let __lto_self_791;
    const a = s;
    const b = __lto_other_794;
    __lto_self_791 = (a + b);
    let __lto_self_789;
    const a_0 = __lto_self_791;
    const b_1 = name;
    __lto_self_789 = (a_0 + b_1);
    const __lto_other_790 = "<'a> {\n";
    let s;
    const a_2 = __lto_self_789;
    const b_3 = __lto_other_790;
    s = (a_2 + b_3);
    return emit_enum_variants(__caps, s, alts, (s) => {
      const __lto_other_802 = "}\n\n";
      let s_0;
      const a = s;
      const b = __lto_other_802;
      s_0 = (a + b);
      const __lto_other_810 = "impl<'a> AstNode<'a> for ";
      let __lto_self_807;
      const a_1 = s_0;
      const b_2 = __lto_other_810;
      __lto_self_807 = (a_1 + b_2);
      let __lto_self_805;
      const a_3 = __lto_self_807;
      const b_4 = name;
      __lto_self_805 = (a_3 + b_4);
      const __lto_other_806 = "<'a> {\n";
      let s_5;
      const a_6 = __lto_self_805;
      const b_7 = __lto_other_806;
      s_5 = (a_6 + b_7);
      const __lto_other_818 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
      let s_8;
      const a_9 = s_5;
      const b_10 = __lto_other_818;
      s_8 = (a_9 + b_10);
      const __lto_other_822 = "        None\n";
      let s_11;
      const a_12 = s_8;
      const b_13 = __lto_other_822;
      s_11 = (a_12 + b_13);
      return emit_enum_cast_chain(__caps, s_11, alts, (s) => {
        const __lto_other_826 = "    }\n";
        let s_0;
        const a = s;
        const b = __lto_other_826;
        s_0 = (a + b);
        const __lto_other_830 = "    fn syntax(&self) -> &'a SyntaxNode {\n";
        let s_1;
        const a_2 = s_0;
        const b_3 = __lto_other_830;
        s_1 = (a_2 + b_3);
        const __lto_other_834 = "        match self {\n";
        let s_4;
        const a_5 = s_1;
        const b_6 = __lto_other_834;
        s_4 = (a_5 + b_6);
        return emit_enum_syntax_arms(__caps, s_4, alts, (s) => {
          const __lto_other_838 = "        }\n";
          let s_0;
          const a = s;
          const b = __lto_other_838;
          s_0 = (a + b);
          const __lto_other_842 = "    }\n";
          let s_1;
          const a_2 = s_0;
          const b_3 = __lto_other_842;
          s_1 = (a_2 + b_3);
          const __lto_other_846 = "}\n\n";
          let s_4;
          const a_5 = s_1;
          const b_6 = __lto_other_846;
          s_4 = (a_5 + b_6);
          return __k(s_4);
        });
      });
    });
  });
}

export function emit_enum_variants__lto(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        return emit_enum_variants(__caps, ((__lto_self_849) => {
          const __lto_other_850 = "<'a>),\n";
          const a = __lto_self_849;
          const b = __lto_other_850;
          return (a + b);
        })(((__lto_self_851) => {
          const a = __lto_self_851;
          const b = name;
          return (a + b);
        })(((__lto_self_853) => {
          const __lto_other_854 = "(";
          const a = __lto_self_853;
          const b = __lto_other_854;
          return (a + b);
        })(((__lto_self_855) => {
          const a = __lto_self_855;
          const b = name;
          return (a + b);
        })(((__lto_self_857) => {
          const __lto_other_858 = "    ";
          const a = __lto_self_857;
          const b = __lto_other_858;
          return (a + b);
        })(s))))), rest, __k);
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_enum_cast_chain__lto(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        const __lto_self_875 = "            .or_else(|| ";
        let __lto_self_873;
        const a = __lto_self_875;
        const b = name;
        __lto_self_873 = (a + b);
        const __lto_other_874 = "::cast(node).map(Self::";
        let __lto_self_871;
        const a_0 = __lto_self_873;
        const b_1 = __lto_other_874;
        __lto_self_871 = (a_0 + b_1);
        let __lto_self_869;
        const a_2 = __lto_self_871;
        const b_3 = name;
        __lto_self_869 = (a_2 + b_3);
        const __lto_other_870 = "))\n";
        let line;
        const a_4 = __lto_self_869;
        const b_5 = __lto_other_870;
        line = (a_4 + b_5);
        return emit_enum_cast_chain(__caps, ((__lto_self_885) => {
          const a = __lto_self_885;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_enum_syntax_arms__lto(__caps, s, alts, __k) {
  return __thunk(() => {
    if ((alts[LUMO_TAG] === "nil")) {
      return __k(s);
    } else if ((alts[LUMO_TAG] === "cons")) {
      const alt = alts.args[0];
      const rest = alts.args[1];
      if ((alt[LUMO_TAG] === "mk")) {
        const name = alt.args[0];
        const __lto_self_891 = "            Self::";
        let __lto_self_889;
        const a = __lto_self_891;
        const b = name;
        __lto_self_889 = (a + b);
        const __lto_other_890 = "(n) => n.syntax(),\n";
        let line;
        const a_0 = __lto_self_889;
        const b_1 = __lto_other_890;
        line = (a_0 + b_1);
        return emit_enum_syntax_arms(__caps, ((__lto_self_897) => {
          const a = __lto_self_897;
          const b = line;
          return (a + b);
        })(s), rest, __k);
      } else {
        return __lumo_match_error(alt);
      }
    } else {
      return __lumo_match_error(alts);
    }
  });
}

export function emit_token_wrapper_node__lto(__caps, s, name, __k) {
  return to_screaming_snake(__caps, name, (kind) => {
    const __lto_other_906 = "pub struct ";
    let __lto_self_903;
    const a = s;
    const b = __lto_other_906;
    __lto_self_903 = (a + b);
    let __lto_self_901;
    const a_0 = __lto_self_903;
    const b_1 = name;
    __lto_self_901 = (a_0 + b_1);
    const __lto_other_902 = "<'a>(pub(crate) &'a SyntaxNode);\n\n";
    let s;
    const a_2 = __lto_self_901;
    const b_3 = __lto_other_902;
    s = (a_2 + b_3);
    const __lto_other_918 = "impl<'a> AstNode<'a> for ";
    let __lto_self_915;
    const a_4 = s;
    const b_5 = __lto_other_918;
    __lto_self_915 = (a_4 + b_5);
    let __lto_self_913;
    const a_6 = __lto_self_915;
    const b_7 = name;
    __lto_self_913 = (a_6 + b_7);
    const __lto_other_914 = "<'a> {\n";
    let s_8;
    const a_9 = __lto_self_913;
    const b_10 = __lto_other_914;
    s_8 = (a_9 + b_10);
    const __lto_other_926 = "    fn cast(node: &'a SyntaxNode) -> Option<Self> {\n";
    let s_11;
    const a_12 = s_8;
    const b_13 = __lto_other_926;
    s_11 = (a_12 + b_13);
    const __lto_other_934 = "        (node.kind == SyntaxKind::";
    let __lto_self_931;
    const a_14 = s_11;
    const b_15 = __lto_other_934;
    __lto_self_931 = (a_14 + b_15);
    let __lto_self_929;
    const a_16 = __lto_self_931;
    const b_17 = kind;
    __lto_self_929 = (a_16 + b_17);
    const __lto_other_930 = ").then(|| Self(node))\n";
    let s_18;
    const a_19 = __lto_self_929;
    const b_20 = __lto_other_930;
    s_18 = (a_19 + b_20);
    const __lto_other_942 = "    }\n";
    let s_21;
    const a_22 = s_18;
    const b_23 = __lto_other_942;
    s_21 = (a_22 + b_23);
    const __lto_other_946 = "    fn syntax(&self) -> &'a SyntaxNode { self.0 }\n";
    let s_24;
    const a_25 = s_21;
    const b_26 = __lto_other_946;
    s_24 = (a_25 + b_26);
    const __lto_other_950 = "}\n\n";
    let s_27;
    const a_28 = s_24;
    const b_29 = __lto_other_950;
    s_27 = (a_28 + b_29);
    return __k(s_27);
  });
}

export function run__lto(__caps, __k) {
  return __thunk(() => {
    let __match_349;
    let __match_354;
    let __lto_self_953;
    const __lto___lto_self_1221_1225 = __argv_length_raw();
    __lto_self_953 = ((__lto___lto_other_1222_1226) => {
      const a = __lto___lto_self_1221_1225;
      const b = __lto___lto_other_1222_1226;
      return (a - b);
    })(1);
    __match_354 = ((__lto_other_954) => {
      let __match_352;
      const a = __lto_self_953;
      const b = __lto_other_954;
      __match_352 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_352[LUMO_TAG] === "true")) {
        return Ordering["less"];
      } else if ((__match_352[LUMO_TAG] === "false")) {
        let __match_353;
        const a = __lto_self_953;
        const b = __lto_other_954;
        if ((a === b)) {
          __match_353 = Bool["true"];
        } else {
          __match_353 = Bool["false"];
        }
        if ((__match_353[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_353[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_353);
        }
      } else {
        return __lumo_match_error(__match_352);
      }
    })(2);
    __match_349 = ((__match_354[LUMO_TAG] === "less") ? Bool["true"] : ((__match_354[LUMO_TAG] === "equal") ? Bool["false"] : ((__match_354[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_354))));
    if ((__match_349[LUMO_TAG] === "true")) {
      const __lto_msg_957 = "Usage: langue <input.langue> [output_dir]";
      const __lto__err_958 = __console_error(__lto_msg_957);
      return __k(__exit_process(1));
    } else if ((__match_349[LUMO_TAG] === "false")) {
      const __lto_idx_959 = 1;
      const file = __argv_at_raw(((__lto___lto_self_1217_1230) => {
        const __lto___lto_other_1218_1231 = 1;
        const a = __lto___lto_self_1217_1230;
        const b = __lto___lto_other_1218_1231;
        return (a + b);
      })(__lto_idx_959));
      const src = readFileSync(file, "utf8");
      return parse_grammar(__caps, src, (__cps_v_270) => {
        if ((__cps_v_270[LUMO_TAG] === "ok")) {
          const raw_grammar = __cps_v_270.args[0];
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
        } else if ((__cps_v_270[LUMO_TAG] === "err")) {
          const msg = __cps_v_270.args[0];
          const pos = __cps_v_270.args[1];
          const __lto_self_967 = "Parse error at position ";
          return Number.to_string(__caps, pos, (__lto_other_968) => {
            let __lto_self_965;
            const a = __lto_self_967;
            const b = __lto_other_968;
            __lto_self_965 = (a + b);
            const __lto_other_966 = ": ";
            let __lto_self_963;
            const a_0 = __lto_self_965;
            const b_1 = __lto_other_966;
            __lto_self_963 = (a_0 + b_1);
            let __lto_msg_961;
            const a_2 = __lto_self_963;
            const b_3 = msg;
            __lto_msg_961 = (a_2 + b_3);
            const __lto__err_962 = __console_error(__lto_msg_961);
            return __k(__exit_process(1));
          });
        } else {
          return __lumo_match_error(__cps_v_270);
        }
      });
    } else {
      return __lumo_match_error(__match_349);
    }
  });
}

export function write_output__lto(out_dir, file, count, syntax_kind_code, ast_code) {
  let sk_path;
  sk_path = ((__lto_other_981) => {
    const a = out_dir;
    const b = __lto_other_981;
    return (a + b);
  })("/syntax_kind.rs");
  let ast_path;
  ast_path = ((__lto_other_985) => {
    const a = out_dir;
    const b = __lto_other_985;
    return (a + b);
  })("/ast.rs");
  let w1;
  w1 = ((__lto_content_989) => {
    return writeFileSync(sk_path, __lto_content_989, "utf8");
  })(syntax_kind_code);
  let w2;
  w2 = ((__lto_content_991) => {
    return writeFileSync(ast_path, __lto_content_991, "utf8");
  })(ast_code);
  let p1;
  let __lto_msg_992;
  let __lto_self_993;
  let __lto_self_995;
  const __lto_self_997 = "Parsed ";
  __lto_self_995 = ((__lto_other_998) => {
    const a = __lto_self_997;
    const b = __lto_other_998;
    return (a + b);
  })(Number.to_string(count));
  __lto_self_993 = ((__lto_other_996) => {
    const a = __lto_self_995;
    const b = __lto_other_996;
    return (a + b);
  })(" rules from ");
  __lto_msg_992 = ((__lto_other_994) => {
    const a = __lto_self_993;
    const b = __lto_other_994;
    return (a + b);
  })(file);
  p1 = ((msg) => {
    return globalThis.console.log(msg);
  })(__lto_msg_992);
  let p2;
  let __lto_msg_1005;
  const __lto_self_1006 = "Wrote ";
  __lto_msg_1005 = ((__lto_other_1007) => {
    const a = __lto_self_1006;
    const b = __lto_other_1007;
    return (a + b);
  })(sk_path);
  p2 = ((msg) => {
    return globalThis.console.log(msg);
  })(__lto_msg_1005);
  let __lto_msg_1010;
  const __lto_self_1011 = "Wrote ";
  __lto_msg_1010 = ((__lto_other_1012) => {
    const a = __lto_self_1011;
    const b = __lto_other_1012;
    return (a + b);
  })(ast_path);
  const msg = __lto_msg_1010;
  return globalThis.console.log(msg);
}

export function list_length_rules__lto(__caps, xs, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(0);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const rest = xs.args[1];
      const __lto_self_1015 = 1;
      return list_length_rules(__caps, rest, (__lto_other_1016) => {
        return __k(((a, b) => {
          return (a + b);
        })(__lto_self_1015, __lto_other_1016));
      });
    } else {
      return __lumo_match_error(xs);
    }
  });
}

export function is_whitespace__lto(c) {
  let __match_356;
  __match_356 = ((__lto_other_1020) => {
    const a = c;
    const b = __lto_other_1020;
    if ((a === b)) {
      return Bool["true"];
    } else {
      return Bool["false"];
    }
  })(" ");
  if ((__match_356[LUMO_TAG] === "true")) {
    return Bool["true"];
  } else if ((__match_356[LUMO_TAG] === "false")) {
    let __match_357;
    const __lto_other_1024 = "\n";
    const a = c;
    const b = __lto_other_1024;
    if ((a === b)) {
      __match_357 = Bool["true"];
    } else {
      __match_357 = Bool["false"];
    }
    if ((__match_357[LUMO_TAG] === "true")) {
      return Bool["true"];
    } else if ((__match_357[LUMO_TAG] === "false")) {
      let __match_358;
      const __lto_other_1028 = "\t";
      const a = c;
      const b = __lto_other_1028;
      if ((a === b)) {
        __match_358 = Bool["true"];
      } else {
        __match_358 = Bool["false"];
      }
      if ((__match_358[LUMO_TAG] === "true")) {
        return Bool["true"];
      } else if ((__match_358[LUMO_TAG] === "false")) {
        let __match_359;
        const __lto_other_1032 = "\r";
        const a = c;
        const b = __lto_other_1032;
        if ((a === b)) {
          __match_359 = Bool["true"];
        } else {
          __match_359 = Bool["false"];
        }
        if ((__match_359[LUMO_TAG] === "true")) {
          return Bool["true"];
        } else if ((__match_359[LUMO_TAG] === "false")) {
          return Bool["false"];
        } else {
          return __lumo_match_error(__match_359);
        }
      } else {
        return __lumo_match_error(__match_358);
      }
    } else {
      return __lumo_match_error(__match_357);
    }
  } else {
    return __lumo_match_error(__match_356);
  }
}

export function is_alpha__lto(c) {
  const code = String.char_code_at(c, 0);
  let __match_363;
  let __match_362;
  __match_362 = ((__lto_other_1036) => {
    let __match_360;
    const a = code;
    const b = __lto_other_1036;
    __match_360 = ((a < b) ? Bool["true"] : Bool["false"]);
    if ((__match_360[LUMO_TAG] === "true")) {
      return Ordering["less"];
    } else if ((__match_360[LUMO_TAG] === "false")) {
      let __match_361;
      const a = code;
      const b = __lto_other_1036;
      if ((a === b)) {
        __match_361 = Bool["true"];
      } else {
        __match_361 = Bool["false"];
      }
      if ((__match_361[LUMO_TAG] === "true")) {
        return Ordering["equal"];
      } else if ((__match_361[LUMO_TAG] === "false")) {
        return Ordering["greater"];
      } else {
        return __lumo_match_error(__match_361);
      }
    } else {
      return __lumo_match_error(__match_360);
    }
  })(97);
  __match_363 = ((__match_362[LUMO_TAG] === "less") ? Bool["false"] : ((__match_362[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_362[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_362))));
  if ((__match_363[LUMO_TAG] === "true")) {
    let __match_373;
    const __lto_other_1040 = 122;
    let __match_371;
    const a = code;
    const b = __lto_other_1040;
    __match_371 = ((a < b) ? Bool["true"] : Bool["false"]);
    if ((__match_371[LUMO_TAG] === "true")) {
      __match_373 = Ordering["less"];
    } else if ((__match_371[LUMO_TAG] === "false")) {
      __match_373 = ((__match_372) => {
        if ((__match_372[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_372[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_372);
        }
      })(((a, b) => {
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })(code, __lto_other_1040));
    } else {
      __match_373 = __lumo_match_error(__match_371);
    }
    if ((__match_373[LUMO_TAG] === "less")) {
      return Bool["true"];
    } else if ((__match_373[LUMO_TAG] === "equal")) {
      return Bool["true"];
    } else {
      return ((__match_373[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_373));
    }
  } else if ((__match_363[LUMO_TAG] === "false")) {
    let __match_367;
    let __match_366;
    const __lto_other_1044 = 65;
    let __match_364;
    const a = code;
    const b = __lto_other_1044;
    __match_364 = ((a < b) ? Bool["true"] : Bool["false"]);
    if ((__match_364[LUMO_TAG] === "true")) {
      __match_366 = Ordering["less"];
    } else if ((__match_364[LUMO_TAG] === "false")) {
      __match_366 = ((__match_365) => {
        if ((__match_365[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_365[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_365);
        }
      })(((a, b) => {
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })(code, __lto_other_1044));
    } else {
      __match_366 = __lumo_match_error(__match_364);
    }
    if ((__match_366[LUMO_TAG] === "less")) {
      __match_367 = Bool["false"];
    } else if ((__match_366[LUMO_TAG] === "equal")) {
      __match_367 = Bool["true"];
    } else {
      __match_367 = ((__match_366[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_366));
    }
    if ((__match_367[LUMO_TAG] === "true")) {
      let __match_370;
      const __lto_other_1048 = 90;
      let __match_368;
      const a = code;
      const b = __lto_other_1048;
      __match_368 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_368[LUMO_TAG] === "true")) {
        __match_370 = Ordering["less"];
      } else if ((__match_368[LUMO_TAG] === "false")) {
        __match_370 = ((__match_369) => {
          if ((__match_369[LUMO_TAG] === "true")) {
            return Ordering["equal"];
          } else if ((__match_369[LUMO_TAG] === "false")) {
            return Ordering["greater"];
          } else {
            return __lumo_match_error(__match_369);
          }
        })(((a, b) => {
          if ((a === b)) {
            return Bool["true"];
          } else {
            return Bool["false"];
          }
        })(code, __lto_other_1048));
      } else {
        __match_370 = __lumo_match_error(__match_368);
      }
      if ((__match_370[LUMO_TAG] === "less")) {
        return Bool["true"];
      } else if ((__match_370[LUMO_TAG] === "equal")) {
        return Bool["true"];
      } else {
        return ((__match_370[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_370));
      }
    } else if ((__match_367[LUMO_TAG] === "false")) {
      return Bool["false"];
    } else {
      return __lumo_match_error(__match_367);
    }
  } else {
    return __lumo_match_error(__match_363);
  }
}

export function is_ident_continue__lto(__caps, c, __k) {
  return is_alpha(__caps, c, (__cps_v_271) => {
    if ((__cps_v_271[LUMO_TAG] === "true")) {
      return __k(Bool["true"]);
    } else if ((__cps_v_271[LUMO_TAG] === "false")) {
      let __match_375;
      __match_375 = ((__lto_other_1052) => {
        const a = c;
        const b = __lto_other_1052;
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })("_");
      if ((__match_375[LUMO_TAG] === "true")) {
        return __k(Bool["true"]);
      } else if ((__match_375[LUMO_TAG] === "false")) {
        return __k(Bool["false"]);
      } else {
        return __lumo_match_error(__match_375);
      }
    } else {
      return __lumo_match_error(__cps_v_271);
    }
  });
}

export function state_eof__lto(st) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    let __match_379;
    __match_379 = ((__lto_other_1056) => {
      let __match_377;
      const a = pos;
      const b = __lto_other_1056;
      __match_377 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_377[LUMO_TAG] === "true")) {
        return Ordering["less"];
      } else if ((__match_377[LUMO_TAG] === "false")) {
        let __match_378;
        const a = pos;
        const b = __lto_other_1056;
        if ((a === b)) {
          __match_378 = Bool["true"];
        } else {
          __match_378 = Bool["false"];
        }
        if ((__match_378[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_378[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_378);
        }
      } else {
        return __lumo_match_error(__match_377);
      }
    })(String.len(src));
    if ((__match_379[LUMO_TAG] === "less")) {
      return Bool["false"];
    } else if ((__match_379[LUMO_TAG] === "equal")) {
      return Bool["true"];
    } else {
      return ((__match_379[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_379));
    }
  } else {
    return __lumo_match_error(st);
  }
}

export function state_peek__lto(st) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    let __match_384;
    let __match_383;
    __match_383 = ((__lto_other_1060) => {
      let __match_381;
      const a = pos;
      const b = __lto_other_1060;
      __match_381 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_381[LUMO_TAG] === "true")) {
        return Ordering["less"];
      } else if ((__match_381[LUMO_TAG] === "false")) {
        let __match_382;
        const a = pos;
        const b = __lto_other_1060;
        if ((a === b)) {
          __match_382 = Bool["true"];
        } else {
          __match_382 = Bool["false"];
        }
        if ((__match_382[LUMO_TAG] === "true")) {
          return Ordering["equal"];
        } else if ((__match_382[LUMO_TAG] === "false")) {
          return Ordering["greater"];
        } else {
          return __lumo_match_error(__match_382);
        }
      } else {
        return __lumo_match_error(__match_381);
      }
    })(String.len(src));
    __match_384 = ((__match_383[LUMO_TAG] === "less") ? Bool["true"] : ((__match_383[LUMO_TAG] === "equal") ? Bool["false"] : ((__match_383[LUMO_TAG] === "greater") ? Bool["false"] : __lumo_match_error(__match_383))));
    if ((__match_384[LUMO_TAG] === "true")) {
      return String.char_at(src, pos);
    } else if ((__match_384[LUMO_TAG] === "false")) {
      return "";
    } else {
      return __lumo_match_error(__match_384);
    }
  } else {
    return __lumo_match_error(st);
  }
}

export function state_advance__lto(st, n) {
  if ((st[LUMO_TAG] === "mk")) {
    const src = st.args[0];
    const pos = st.args[1];
    return ParseState["mk"](src, ((__lto_self_1063) => {
      const a = __lto_self_1063;
      const b = n;
      return (a + b);
    })(pos));
  } else {
    return __lumo_match_error(st);
  }
}

export function skip_ws__lto(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_279) => {
    if ((__cps_v_279[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_279[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (c) => {
        return is_whitespace(__caps, c, (__cps_v_278) => {
          if ((__cps_v_278[LUMO_TAG] === "true")) {
            return state_advance(__caps, st, 1, (__cps_v_277) => {
              return skip_ws(__caps, __cps_v_277, __k);
            });
          } else if ((__cps_v_278[LUMO_TAG] === "false")) {
            let __match_388;
            __match_388 = ((__lto_other_1068) => {
              const a = c;
              const b = __lto_other_1068;
              if ((a === b)) {
                return Bool["true"];
              } else {
                return Bool["false"];
              }
            })("/");
            if ((__match_388[LUMO_TAG] === "true")) {
              const __lto_self_1071 = state_pos(st);
              const __lto_other_1072 = 1;
              let next_pos;
              const a = __lto_self_1071;
              const b = __lto_other_1072;
              next_pos = (a + b);
              return String.len(__caps, state_src(st), (__lto_other_1076) => {
                const __k_271 = (__cps_v_276) => {
                  const __k_270 = (__cps_v_275) => {
                    if ((__cps_v_275[LUMO_TAG] === "true")) {
                      return String.char_at(__caps, state_src(st), next_pos, (__lto_self_1079) => {
                        const __lto_other_1080 = "/";
                        let __cps_v_274;
                        const a = __lto_self_1079;
                        const b = __lto_other_1080;
                        __cps_v_274 = ((a === b) ? Bool["true"] : Bool["false"]);
                        if ((__cps_v_274[LUMO_TAG] === "true")) {
                          return state_advance(__caps, st, 2, (__cps_v_273) => {
                            return skip_line(__caps, __cps_v_273, (__cps_v_272) => {
                              return skip_ws(__caps, __cps_v_272, __k);
                            });
                          });
                        } else if ((__cps_v_274[LUMO_TAG] === "false")) {
                          return __k(st);
                        } else {
                          return __lumo_match_error(__cps_v_274);
                        }
                      });
                    } else if ((__cps_v_275[LUMO_TAG] === "false")) {
                      return __k(st);
                    } else {
                      return __lumo_match_error(__cps_v_275);
                    }
                  };
                  if ((__cps_v_276[LUMO_TAG] === "less")) {
                    return __k_270(Bool["true"]);
                  } else if ((__cps_v_276[LUMO_TAG] === "equal")) {
                    return __k_270(Bool["false"]);
                  } else {
                    return ((__cps_v_276[LUMO_TAG] === "greater") ? __k_270(Bool["false"]) : __lumo_match_error(__cps_v_276));
                  }
                };
                let __match_392;
                const a = next_pos;
                const b = __lto_other_1076;
                __match_392 = ((a < b) ? Bool["true"] : Bool["false"]);
                if ((__match_392[LUMO_TAG] === "true")) {
                  return __k_271(Ordering["less"]);
                } else if ((__match_392[LUMO_TAG] === "false")) {
                  let __match_393;
                  const a = next_pos;
                  const b = __lto_other_1076;
                  __match_393 = ((a === b) ? Bool["true"] : Bool["false"]);
                  if ((__match_393[LUMO_TAG] === "true")) {
                    return __k_271(Ordering["equal"]);
                  } else if ((__match_393[LUMO_TAG] === "false")) {
                    return __k_271(Ordering["greater"]);
                  } else {
                    return __lumo_match_error(__match_393);
                  }
                } else {
                  return __lumo_match_error(__match_392);
                }
              });
            } else if ((__match_388[LUMO_TAG] === "false")) {
              return __k(st);
            } else {
              return __lumo_match_error(__match_388);
            }
          } else {
            return __lumo_match_error(__cps_v_278);
          }
        });
      });
    } else {
      return __lumo_match_error(__cps_v_279);
    }
  });
}

export function skip_line__lto(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_282) => {
    if ((__cps_v_282[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_282[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__lto_self_1083) => {
        const __lto_other_1084 = "\n";
        let __cps_v_281;
        const a = __lto_self_1083;
        const b = __lto_other_1084;
        __cps_v_281 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__cps_v_281[LUMO_TAG] === "true")) {
          return state_advance(__caps, st, 1, __k);
        } else if ((__cps_v_281[LUMO_TAG] === "false")) {
          return state_advance(__caps, st, 1, (__cps_v_280) => {
            return skip_line(__caps, __cps_v_280, __k);
          });
        } else {
          return __lumo_match_error(__cps_v_281);
        }
      });
    } else {
      return __lumo_match_error(__cps_v_282);
    }
  });
}

export function parse_ident__lto(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_288) => {
      if ((__cps_v_288[LUMO_TAG] === "true")) {
        return __k(ParseResult["err"]("expected identifier, got EOF", state_pos(st2)));
      } else if ((__cps_v_288[LUMO_TAG] === "false")) {
        return state_peek(__caps, st2, (__cps_v_287) => {
          return is_ident_start(__caps, __cps_v_287, (__cps_v_286) => {
            if ((__cps_v_286[LUMO_TAG] === "true")) {
              const start = state_pos(st2);
              return state_advance(__caps, st2, 1, (__cps_v_285) => {
                return scan_ident_rest(__caps, __cps_v_285, (end_st) => {
                  const end_pos = state_pos(end_st);
                  return String.slice(__caps, state_src(st2), start, end_pos, (__cps_v_284) => {
                    return __k(ParseResult["ok"](__cps_v_284, end_st));
                  });
                });
              });
            } else if ((__cps_v_286[LUMO_TAG] === "false")) {
              const __lto_self_1089 = "expected identifier, got '";
              return state_peek(__caps, st2, (__lto_other_1090) => {
                let __lto_self_1087;
                const a = __lto_self_1089;
                const b = __lto_other_1090;
                __lto_self_1087 = (a + b);
                const __lto_other_1088 = "'";
                let __cps_v_283;
                const a_0 = __lto_self_1087;
                const b_1 = __lto_other_1088;
                __cps_v_283 = (a_0 + b_1);
                return __k(ParseResult["err"](__cps_v_283, state_pos(st2)));
              });
            } else {
              return __lumo_match_error(__cps_v_286);
            }
          });
        });
      } else {
        return __lumo_match_error(__cps_v_288);
      }
    });
  });
}

export function expect__lto(__caps, st, expected, __k) {
  return skip_ws(__caps, st, (st2) => {
    return String.len(__caps, expected, (len) => {
      const src = state_src(st2);
      const pos = state_pos(st2);
      return String.len(__caps, src, (__lto_self_1095) => {
        let remaining;
        const a = __lto_self_1095;
        const b = pos;
        remaining = (a - b);
        let __match_398;
        let __match_402;
        __match_402 = ((__lto_other_1100) => {
          let __match_400;
          const a = remaining;
          const b = __lto_other_1100;
          __match_400 = ((a < b) ? Bool["true"] : Bool["false"]);
          if ((__match_400[LUMO_TAG] === "true")) {
            return Ordering["less"];
          } else if ((__match_400[LUMO_TAG] === "false")) {
            let __match_401;
            const a = remaining;
            const b = __lto_other_1100;
            if ((a === b)) {
              __match_401 = Bool["true"];
            } else {
              __match_401 = Bool["false"];
            }
            if ((__match_401[LUMO_TAG] === "true")) {
              return Ordering["equal"];
            } else if ((__match_401[LUMO_TAG] === "false")) {
              return Ordering["greater"];
            } else {
              return __lumo_match_error(__match_401);
            }
          } else {
            return __lumo_match_error(__match_400);
          }
        })(len);
        __match_398 = ((__match_402[LUMO_TAG] === "less") ? Bool["false"] : ((__match_402[LUMO_TAG] === "equal") ? Bool["true"] : ((__match_402[LUMO_TAG] === "greater") ? Bool["true"] : __lumo_match_error(__match_402))));
        if ((__match_398[LUMO_TAG] === "true")) {
          return String.slice(__caps, src, pos, ((__lto_self_1103) => {
            const a = __lto_self_1103;
            const b = len;
            return (a + b);
          })(pos), (slice) => {
            let __match_399;
            __match_399 = ((__lto_other_1108) => {
              const a = slice;
              const b = __lto_other_1108;
              if ((a === b)) {
                return Bool["true"];
              } else {
                return Bool["false"];
              }
            })(expected);
            if ((__match_399[LUMO_TAG] === "true")) {
              return state_advance(__caps, st2, len, (__cps_v_289) => {
                return __k(ParseResult["ok"](expected, __cps_v_289));
              });
            } else if ((__match_399[LUMO_TAG] === "false")) {
              return __k(ParseResult["err"](((__lto_self_1111) => {
                const __lto_other_1112 = "'";
                const a = __lto_self_1111;
                const b = __lto_other_1112;
                return (a + b);
              })(((__lto_self_1113) => {
                const a = __lto_self_1113;
                const b = slice;
                return (a + b);
              })(((__lto_self_1115) => {
                const __lto_other_1116 = "', got '";
                const a = __lto_self_1115;
                const b = __lto_other_1116;
                return (a + b);
              })(((__lto_self_1117) => {
                const a = __lto_self_1117;
                const b = expected;
                return (a + b);
              })("expected '")))), pos));
            } else {
              return __lumo_match_error(__match_399);
            }
          });
        } else if ((__match_398[LUMO_TAG] === "false")) {
          return __k(ParseResult["err"](((__lto_self_1127) => {
            const __lto_other_1128 = "'";
            const a = __lto_self_1127;
            const b = __lto_other_1128;
            return (a + b);
          })(((__lto_self_1129) => {
            const a = __lto_self_1129;
            const b = expected;
            return (a + b);
          })("expected '")), pos));
        } else {
          return __lumo_match_error(__match_398);
        }
      });
    });
  });
}

export function parse_quoted__lto(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__lto_self_1135) => {
      const __lto_other_1136 = "'";
      let __cps_v_292;
      const a = __lto_self_1135;
      const b = __lto_other_1136;
      __cps_v_292 = ((a === b) ? Bool["true"] : Bool["false"]);
      if ((__cps_v_292[LUMO_TAG] === "true")) {
        const __lto_self_1139 = state_pos(st2);
        const __lto_other_1140 = 1;
        let start;
        const a = __lto_self_1139;
        const b = __lto_other_1140;
        start = (a + b);
        return state_advance(__caps, st2, 1, (__cps_v_291) => {
          return scan_until_quote(__caps, __cps_v_291, (end_st) => {
            const end_pos = state_pos(end_st);
            return String.slice(__caps, state_src(st2), start, end_pos, (content) => {
              return state_advance(__caps, end_st, 1, (__cps_v_290) => {
                return __k(ParseResult["ok"](content, __cps_v_290));
              });
            });
          });
        });
      } else if ((__cps_v_292[LUMO_TAG] === "false")) {
        return __k(ParseResult["err"]("expected quoted literal", state_pos(st2)));
      } else {
        return __lumo_match_error(__cps_v_292);
      }
    });
  });
}

export function scan_until_quote__lto(__caps, st, __k) {
  return state_eof(__caps, st, (__cps_v_295) => {
    if ((__cps_v_295[LUMO_TAG] === "true")) {
      return __k(st);
    } else if ((__cps_v_295[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__lto_self_1143) => {
        const __lto_other_1144 = "'";
        let __cps_v_294;
        const a = __lto_self_1143;
        const b = __lto_other_1144;
        __cps_v_294 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__cps_v_294[LUMO_TAG] === "true")) {
          return __k(st);
        } else if ((__cps_v_294[LUMO_TAG] === "false")) {
          return state_advance(__caps, st, 1, (__cps_v_293) => {
            return scan_until_quote(__caps, __cps_v_293, __k);
          });
        } else {
          return __lumo_match_error(__cps_v_294);
        }
      });
    } else {
      return __lumo_match_error(__cps_v_295);
    }
  });
}

export function peek_is_rule_start__lto(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__cps_v_298) => {
      return is_ident_start(__caps, __cps_v_298, (__cps_v_297) => {
        if ((__cps_v_297[LUMO_TAG] === "true")) {
          return state_advance(__caps, st2, 1, (__cps_v_296) => {
            return scan_ident_rest(__caps, __cps_v_296, (st3) => {
              return skip_ws(__caps, st3, (st4) => {
                return state_peek(__caps, st4, (__lto_self_1147) => {
                  const __lto_other_1148 = "=";
                  return __k(((a, b) => {
                    if ((a === b)) {
                      return Bool["true"];
                    } else {
                      return Bool["false"];
                    }
                  })(__lto_self_1147, __lto_other_1148));
                });
              });
            });
          });
        } else if ((__cps_v_297[LUMO_TAG] === "false")) {
          return __k(Bool["false"]);
        } else {
          return __lumo_match_error(__cps_v_297);
        }
      });
    });
  });
}

export function has_alpha__lto(__caps, s, i, __k) {
  return __thunk(() => {
    return String.len(__caps, s, (__lto_other_1152) => {
      const __k_286 = (__cps_v_302) => {
        const __k_285 = (__cps_v_301) => {
          if ((__cps_v_301[LUMO_TAG] === "true")) {
            return __k(Bool["false"]);
          } else if ((__cps_v_301[LUMO_TAG] === "false")) {
            return String.char_at(__caps, s, i, (__cps_v_300) => {
              return is_alpha(__caps, __cps_v_300, (__cps_v_299) => {
                if ((__cps_v_299[LUMO_TAG] === "true")) {
                  return __k(Bool["true"]);
                } else if ((__cps_v_299[LUMO_TAG] === "false")) {
                  return has_alpha(__caps, s, ((__lto_self_1155) => {
                    const __lto_other_1156 = 1;
                    const a = __lto_self_1155;
                    const b = __lto_other_1156;
                    return (a + b);
                  })(i), __k);
                } else {
                  return __lumo_match_error(__cps_v_299);
                }
              });
            });
          } else {
            return __lumo_match_error(__cps_v_301);
          }
        };
        if ((__cps_v_302[LUMO_TAG] === "less")) {
          return __k_285(Bool["false"]);
        } else if ((__cps_v_302[LUMO_TAG] === "equal")) {
          return __k_285(Bool["true"]);
        } else {
          return ((__cps_v_302[LUMO_TAG] === "greater") ? __k_285(Bool["true"]) : __lumo_match_error(__cps_v_302));
        }
      };
      let __match_410;
      const a = i;
      const b = __lto_other_1152;
      __match_410 = ((a < b) ? Bool["true"] : Bool["false"]);
      if ((__match_410[LUMO_TAG] === "true")) {
        return __k_286(Ordering["less"]);
      } else if ((__match_410[LUMO_TAG] === "false")) {
        let __match_411;
        const a = i;
        const b = __lto_other_1152;
        __match_411 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__match_411[LUMO_TAG] === "true")) {
          return __k_286(Ordering["equal"]);
        } else if ((__match_411[LUMO_TAG] === "false")) {
          return __k_286(Ordering["greater"]);
        } else {
          return __lumo_match_error(__match_411);
        }
      } else {
        return __lumo_match_error(__match_410);
      }
    });
  });
}

export function parse_grammar_items__lto(__caps, st, tokens, rules, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_306) => {
      if ((__cps_v_306[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](Grammar["mk"](list_reverse_string(tokens), list_reverse_rule(rules)), st2));
      } else if ((__cps_v_306[LUMO_TAG] === "false")) {
        return state_peek(__caps, st2, (__lto_self_1159) => {
          const __lto_other_1160 = "@";
          let __cps_v_305;
          const a = __lto_self_1159;
          const b = __lto_other_1160;
          __cps_v_305 = ((a === b) ? Bool["true"] : Bool["false"]);
          if ((__cps_v_305[LUMO_TAG] === "true")) {
            return parse_token_def(__caps, st2, (__cps_v_304) => {
              if ((__cps_v_304[LUMO_TAG] === "ok")) {
                const new_tokens = __cps_v_304.args[0];
                const st3 = __cps_v_304.args[1];
                return parse_grammar_items(__caps, st3, list_concat_string(new_tokens, tokens), rules, __k);
              } else if ((__cps_v_304[LUMO_TAG] === "err")) {
                const msg = __cps_v_304.args[0];
                const pos = __cps_v_304.args[1];
                return __k(ParseResult["err"](msg, pos));
              } else {
                return __lumo_match_error(__cps_v_304);
              }
            });
          } else if ((__cps_v_305[LUMO_TAG] === "false")) {
            return parse_rule(__caps, st2, (__cps_v_303) => {
              if ((__cps_v_303[LUMO_TAG] === "ok")) {
                const rule = __cps_v_303.args[0];
                const st3 = __cps_v_303.args[1];
                return parse_grammar_items(__caps, st3, tokens, List["cons"](rule, rules), __k);
              } else if ((__cps_v_303[LUMO_TAG] === "err")) {
                const msg = __cps_v_303.args[0];
                const pos = __cps_v_303.args[1];
                return __k(ParseResult["err"](msg, pos));
              } else {
                return __lumo_match_error(__cps_v_303);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_305);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_306);
      }
    });
  });
}

export function parse_token_names__lto(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_eof(__caps, st2, (__cps_v_312) => {
      if ((__cps_v_312[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](list_reverse_string(acc), st2));
      } else if ((__cps_v_312[LUMO_TAG] === "false")) {
        return peek_is_rule_start(__caps, st2, (__cps_v_311) => {
          if ((__cps_v_311[LUMO_TAG] === "true")) {
            return __k(ParseResult["ok"](list_reverse_string(acc), st2));
          } else if ((__cps_v_311[LUMO_TAG] === "false")) {
            return state_peek(__caps, st2, (__lto_self_1163) => {
              const __lto_other_1164 = "@";
              let __cps_v_310;
              const a = __lto_self_1163;
              const b = __lto_other_1164;
              __cps_v_310 = ((a === b) ? Bool["true"] : Bool["false"]);
              if ((__cps_v_310[LUMO_TAG] === "true")) {
                return __k(ParseResult["ok"](list_reverse_string(acc), st2));
              } else if ((__cps_v_310[LUMO_TAG] === "false")) {
                return state_peek(__caps, st2, (__cps_v_309) => {
                  return is_ident_start(__caps, __cps_v_309, (__cps_v_308) => {
                    if ((__cps_v_308[LUMO_TAG] === "true")) {
                      return parse_ident(__caps, st2, (__cps_v_307) => {
                        if ((__cps_v_307[LUMO_TAG] === "ok")) {
                          const name = __cps_v_307.args[0];
                          const st3 = __cps_v_307.args[1];
                          return parse_token_names(__caps, st3, List["cons"](name, acc), __k);
                        } else if ((__cps_v_307[LUMO_TAG] === "err")) {
                          const msg = __cps_v_307.args[0];
                          const pos = __cps_v_307.args[1];
                          return __k(ParseResult["err"](msg, pos));
                        } else {
                          return __lumo_match_error(__cps_v_307);
                        }
                      });
                    } else if ((__cps_v_308[LUMO_TAG] === "false")) {
                      return __k(ParseResult["ok"](list_reverse_string(acc), st2));
                    } else {
                      return __lumo_match_error(__cps_v_308);
                    }
                  });
                });
              } else {
                return __lumo_match_error(__cps_v_310);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_311);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_312);
      }
    });
  });
}

export function parse_rule_body__lto(__caps, st, rule_name, __k) {
  return skip_ws(__caps, st, (st2) => {
    return peek_char(__caps, st2, (__lto_self_1167) => {
      const __lto_other_1168 = "|";
      let __cps_v_313;
      const a = __lto_self_1167;
      const b = __lto_other_1168;
      __cps_v_313 = ((a === b) ? Bool["true"] : Bool["false"]);
      if ((__cps_v_313[LUMO_TAG] === "true")) {
        return parse_alternatives(__caps, st2, __k);
      } else if ((__cps_v_313[LUMO_TAG] === "false")) {
        return parse_sequence(__caps, st2, __k);
      } else {
        return __lumo_match_error(__cps_v_313);
      }
    });
  });
}

export function parse_alt_items__lto(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return peek_char(__caps, st2, (__lto_self_1171) => {
      const __lto_other_1172 = "|";
      let __cps_v_318;
      const a = __lto_self_1171;
      const b = __lto_other_1172;
      __cps_v_318 = ((a === b) ? Bool["true"] : Bool["false"]);
      if ((__cps_v_318[LUMO_TAG] === "true")) {
        return skip_ws(__caps, st2, (__cps_v_317) => {
          return state_advance(__caps, __cps_v_317, 1, (st3) => {
            return skip_ws(__caps, st3, (st4) => {
              return state_peek(__caps, st4, (__lto_self_1175) => {
                const __lto_other_1176 = "'";
                let __cps_v_316;
                const a = __lto_self_1175;
                const b = __lto_other_1176;
                __cps_v_316 = ((a === b) ? Bool["true"] : Bool["false"]);
                if ((__cps_v_316[LUMO_TAG] === "true")) {
                  return parse_quoted(__caps, st4, (__cps_v_315) => {
                    if ((__cps_v_315[LUMO_TAG] === "ok")) {
                      const lit = __cps_v_315.args[0];
                      const st5 = __cps_v_315.args[1];
                      return parse_alt_items(__caps, st5, List["cons"](Alternative["mk"](lit), acc), __k);
                    } else if ((__cps_v_315[LUMO_TAG] === "err")) {
                      const msg = __cps_v_315.args[0];
                      const pos = __cps_v_315.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_315);
                    }
                  });
                } else if ((__cps_v_316[LUMO_TAG] === "false")) {
                  return parse_ident(__caps, st3, (__cps_v_314) => {
                    if ((__cps_v_314[LUMO_TAG] === "ok")) {
                      const name = __cps_v_314.args[0];
                      const st5 = __cps_v_314.args[1];
                      return parse_alt_items(__caps, st5, List["cons"](Alternative["mk"](name), acc), __k);
                    } else if ((__cps_v_314[LUMO_TAG] === "err")) {
                      const msg = __cps_v_314.args[0];
                      const pos = __cps_v_314.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_314);
                    }
                  });
                } else {
                  return __lumo_match_error(__cps_v_316);
                }
              });
            });
          });
        });
      } else if ((__cps_v_318[LUMO_TAG] === "false")) {
        return __k(ParseResult["ok"](RuleBody["alternatives"](list_reverse_alt(acc)), st2));
      } else {
        return __lumo_match_error(__cps_v_318);
      }
    });
  });
}

export function is_seq_terminator__lto(__caps, st, __k) {
  return peek_char(__caps, st, (c) => {
    let __match_426;
    __match_426 = ((__lto_other_1180) => {
      const a = c;
      const b = __lto_other_1180;
      if ((a === b)) {
        return Bool["true"];
      } else {
        return Bool["false"];
      }
    })(")");
    if ((__match_426[LUMO_TAG] === "true")) {
      return __k(Bool["true"]);
    } else if ((__match_426[LUMO_TAG] === "false")) {
      return peek_is_rule_start(__caps, st, (__cps_v_319) => {
        if ((__cps_v_319[LUMO_TAG] === "true")) {
          return __k(Bool["true"]);
        } else if ((__cps_v_319[LUMO_TAG] === "false")) {
          let __match_428;
          __match_428 = ((__lto_other_1184) => {
            const a = c;
            const b = __lto_other_1184;
            if ((a === b)) {
              return Bool["true"];
            } else {
              return Bool["false"];
            }
          })("@");
          if ((__match_428[LUMO_TAG] === "true")) {
            return __k(Bool["true"]);
          } else if ((__match_428[LUMO_TAG] === "false")) {
            return __k(Bool["false"]);
          } else {
            return __lumo_match_error(__match_428);
          }
        } else {
          return __lumo_match_error(__cps_v_319);
        }
      });
    } else {
      return __lumo_match_error(__match_426);
    }
  });
}

export function apply_postfix_elem__lto(__caps, elem, st, __k) {
  return state_eof(__caps, st, (__cps_v_324) => {
    if ((__cps_v_324[LUMO_TAG] === "true")) {
      return __k(ParseResult["ok"](elem, st));
    } else if ((__cps_v_324[LUMO_TAG] === "false")) {
      return state_peek(__caps, st, (__lto_self_1187) => {
        const __lto_other_1188 = "?";
        let __cps_v_323;
        const a = __lto_self_1187;
        const b = __lto_other_1188;
        __cps_v_323 = ((a === b) ? Bool["true"] : Bool["false"]);
        if ((__cps_v_323[LUMO_TAG] === "true")) {
          return state_advance(__caps, st, 1, (__cps_v_322) => {
            return apply_postfix_elem(__caps, Element["optional"](elem), __cps_v_322, __k);
          });
        } else if ((__cps_v_323[LUMO_TAG] === "false")) {
          return state_peek(__caps, st, (__lto_self_1191) => {
            const __lto_other_1192 = "*";
            let __cps_v_321;
            const a = __lto_self_1191;
            const b = __lto_other_1192;
            __cps_v_321 = ((a === b) ? Bool["true"] : Bool["false"]);
            if ((__cps_v_321[LUMO_TAG] === "true")) {
              return state_advance(__caps, st, 1, (__cps_v_320) => {
                return apply_postfix_elem(__caps, Element["repeated"](elem), __cps_v_320, __k);
              });
            } else if ((__cps_v_321[LUMO_TAG] === "false")) {
              return __k(ParseResult["ok"](elem, st));
            } else {
              return __lumo_match_error(__cps_v_321);
            }
          });
        } else {
          return __lumo_match_error(__cps_v_323);
        }
      });
    } else {
      return __lumo_match_error(__cps_v_324);
    }
  });
}

export function parse_atom__lto(__caps, st, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__lto_self_1195) => {
      const __lto_other_1196 = "'";
      let __cps_v_334;
      const a = __lto_self_1195;
      const b = __lto_other_1196;
      __cps_v_334 = ((a === b) ? Bool["true"] : Bool["false"]);
      if ((__cps_v_334[LUMO_TAG] === "true")) {
        return parse_quoted(__caps, st2, (__cps_v_333) => {
          if ((__cps_v_333[LUMO_TAG] === "ok")) {
            const lit = __cps_v_333.args[0];
            const st3 = __cps_v_333.args[1];
            return classify_literal(__caps, lit, (__cps_v_332) => {
              const __cps_v_331 = Element["token"](__cps_v_332);
              return __k(ParseResult["ok"](__cps_v_331, st3));
            });
          } else if ((__cps_v_333[LUMO_TAG] === "err")) {
            const msg = __cps_v_333.args[0];
            const pos = __cps_v_333.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_333);
          }
        });
      } else if ((__cps_v_334[LUMO_TAG] === "false")) {
        return state_peek(__caps, st2, (__lto_self_1199) => {
          const __lto_other_1200 = "(";
          let __cps_v_330;
          const a = __lto_self_1199;
          const b = __lto_other_1200;
          __cps_v_330 = ((a === b) ? Bool["true"] : Bool["false"]);
          if ((__cps_v_330[LUMO_TAG] === "true")) {
            return state_advance(__caps, st2, 1, (st3) => {
              return parse_group_elements(__caps, st3, List["nil"], (__cps_v_329) => {
                if ((__cps_v_329[LUMO_TAG] === "ok")) {
                  const elems = __cps_v_329.args[0];
                  const st4 = __cps_v_329.args[1];
                  return expect(__caps, st4, ")", (__cps_v_328) => {
                    if ((__cps_v_328[LUMO_TAG] === "ok")) {
                      const st5 = __cps_v_328.args[1];
                      return __k(ParseResult["ok"](Element["group"](elems), st5));
                    } else if ((__cps_v_328[LUMO_TAG] === "err")) {
                      const msg = __cps_v_328.args[0];
                      const pos = __cps_v_328.args[1];
                      return __k(ParseResult["err"](msg, pos));
                    } else {
                      return __lumo_match_error(__cps_v_328);
                    }
                  });
                } else if ((__cps_v_329[LUMO_TAG] === "err")) {
                  const msg = __cps_v_329.args[0];
                  const pos = __cps_v_329.args[1];
                  return __k(ParseResult["err"](msg, pos));
                } else {
                  return __lumo_match_error(__cps_v_329);
                }
              });
            });
          } else if ((__cps_v_330[LUMO_TAG] === "false")) {
            return parse_ident(__caps, st2, (__cps_v_327) => {
              if ((__cps_v_327[LUMO_TAG] === "err")) {
                const msg = __cps_v_327.args[0];
                const pos = __cps_v_327.args[1];
                return __k(ParseResult["err"](msg, pos));
              } else if ((__cps_v_327[LUMO_TAG] === "ok")) {
                const name = __cps_v_327.args[0];
                const st3 = __cps_v_327.args[1];
                return state_peek(__caps, st3, (__lto_self_1203) => {
                  const __lto_other_1204 = ":";
                  let __cps_v_326;
                  const a = __lto_self_1203;
                  const b = __lto_other_1204;
                  __cps_v_326 = ((a === b) ? Bool["true"] : Bool["false"]);
                  if ((__cps_v_326[LUMO_TAG] === "true")) {
                    return state_advance(__caps, st3, 1, (st4) => {
                      return parse_element(__caps, st4, (__cps_v_325) => {
                        if ((__cps_v_325[LUMO_TAG] === "ok")) {
                          const inner = __cps_v_325.args[0];
                          const st5 = __cps_v_325.args[1];
                          return __k(ParseResult["ok"](Element["labeled"](name, inner), st5));
                        } else if ((__cps_v_325[LUMO_TAG] === "err")) {
                          const msg = __cps_v_325.args[0];
                          const pos = __cps_v_325.args[1];
                          return __k(ParseResult["err"](msg, pos));
                        } else {
                          return __lumo_match_error(__cps_v_325);
                        }
                      });
                    });
                  } else if ((__cps_v_326[LUMO_TAG] === "false")) {
                    return __k(ParseResult["ok"](Element["node"](NodeRef["mk"](name)), st3));
                  } else {
                    return __lumo_match_error(__cps_v_326);
                  }
                });
              } else {
                return __lumo_match_error(__cps_v_327);
              }
            });
          } else {
            return __lumo_match_error(__cps_v_330);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_334);
      }
    });
  });
}

export function parse_group_elements__lto(__caps, st, acc, __k) {
  return skip_ws(__caps, st, (st2) => {
    return state_peek(__caps, st2, (__lto_self_1207) => {
      const __lto_other_1208 = ")";
      let __cps_v_336;
      const a = __lto_self_1207;
      const b = __lto_other_1208;
      __cps_v_336 = ((a === b) ? Bool["true"] : Bool["false"]);
      if ((__cps_v_336[LUMO_TAG] === "true")) {
        return __k(ParseResult["ok"](list_reverse_elem(acc), st2));
      } else if ((__cps_v_336[LUMO_TAG] === "false")) {
        return parse_element(__caps, st2, (__cps_v_335) => {
          if ((__cps_v_335[LUMO_TAG] === "ok")) {
            const elem = __cps_v_335.args[0];
            const st3 = __cps_v_335.args[1];
            return parse_group_elements(__caps, st3, List["cons"](elem, acc), __k);
          } else if ((__cps_v_335[LUMO_TAG] === "err")) {
            const msg = __cps_v_335.args[0];
            const pos = __cps_v_335.args[1];
            return __k(ParseResult["err"](msg, pos));
          } else {
            return __lumo_match_error(__cps_v_335);
          }
        });
      } else {
        return __lumo_match_error(__cps_v_336);
      }
    });
  });
}

export function list_contains_string__lto(__caps, xs, target, __k) {
  return __thunk(() => {
    if ((xs[LUMO_TAG] === "nil")) {
      return __k(Bool["false"]);
    } else if ((xs[LUMO_TAG] === "cons")) {
      const x = xs.args[0];
      const rest = xs.args[1];
      let __match_443;
      __match_443 = ((__lto_other_1212) => {
        const a = x;
        const b = __lto_other_1212;
        if ((a === b)) {
          return Bool["true"];
        } else {
          return Bool["false"];
        }
      })(target);
      if ((__match_443[LUMO_TAG] === "true")) {
        return __k(Bool["true"]);
      } else if ((__match_443[LUMO_TAG] === "false")) {
        return list_contains_string(__caps, rest, target, __k);
      } else {
        return __lumo_match_error(__match_443);
      }
    } else {
      return __lumo_match_error(xs);
    }
  });
}

main();
