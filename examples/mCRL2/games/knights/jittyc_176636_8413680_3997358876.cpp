#define INDEX_BOUND__ 497// These values are not used anymore.
#define ARITY_BOUND__ 9// These values are not used anymore.
#include "mcrl2/data/detail/rewrite/jittycpreamble.h"
namespace {
// Anonymous namespace so the compiler uses internal linkage for the generated
// rewrite code.

struct rewr_functions
{
  // A rewrite_term is a term that may or may not be in normal form. If the method
  // normal_form is invoked, it will calculate a normal form for itself as efficiently as possible.
  template <class REWRITE_TERM>
  static data_expression& local_rewrite(const REWRITE_TERM& t)
  {
    return t.normal_form();
  }

  // A rewrite_term is a term that may or may not be in normal form. If the method
  // normal_form is invoked, it will calculate a normal form for itself as efficiently as possible.
  template <class REWRITE_TERM>
  static void local_rewrite(data_expression& result,
                            const REWRITE_TERM& t) 
  {
     t.normal_form(result);
  }

  static const data_expression& local_rewrite(const data_expression& t)
  {
    return t;
  }

  static void local_rewrite(data_expression& result, const data_expression& t)
  {
     result=t;
  }

  template < class HEAD, class DATA_EXPR0 >
  class delayed_application1
  {
    protected:
      const HEAD& m_head;
      const DATA_EXPR0& m_arg0;
      RewriterCompilingJitty* this_rewriter;

    public:
      delayed_application1(const HEAD& head, const DATA_EXPR0& arg0, RewriterCompilingJitty* tr)
        : m_head(head), m_arg0(arg0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        make_application(local_store, [&](data_expression& r){ local_rewrite(r, m_head); }, [&](data_expression& r){ local_rewrite(r, m_arg0); });
        rewrite_with_arguments_in_normal_form(local_store, local_store, this_rewriter);
        return local_store;
      }

      void normal_form(data_expression& result) const
      {
        make_application(result, [&](data_expression& result){ local_rewrite(result, m_head); }, [&](data_expression& result){ local_rewrite(result, m_arg0); });
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
      }

  };

  template < class HEAD, class DATA_EXPR0, class DATA_EXPR1 >
  class delayed_application2
  {
    protected:
      const HEAD& m_head;
      const DATA_EXPR0& m_arg0;
      const DATA_EXPR1& m_arg1;
      RewriterCompilingJitty* this_rewriter;

    public:
      delayed_application2(const HEAD& head, const DATA_EXPR0& arg0, const DATA_EXPR1& arg1, RewriterCompilingJitty* tr)
        : m_head(head), m_arg0(arg0), m_arg1(arg1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        make_application(local_store, [&](data_expression& r){ local_rewrite(r, m_head); }, [&](data_expression& r){ local_rewrite(r, m_arg0); }, [&](data_expression& r){ local_rewrite(r, m_arg1); });
        rewrite_with_arguments_in_normal_form(local_store, local_store, this_rewriter);
        return local_store;
      }

      void normal_form(data_expression& result) const
      {
        make_application(result, [&](data_expression& result){ local_rewrite(result, m_head); }, [&](data_expression& result){ local_rewrite(result, m_arg0); }, [&](data_expression& result){ local_rewrite(result, m_arg1); });
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
      }

  };

  template < class HEAD, class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2 >
  class delayed_application3
  {
    protected:
      const HEAD& m_head;
      const DATA_EXPR0& m_arg0;
      const DATA_EXPR1& m_arg1;
      const DATA_EXPR2& m_arg2;
      RewriterCompilingJitty* this_rewriter;

    public:
      delayed_application3(const HEAD& head, const DATA_EXPR0& arg0, const DATA_EXPR1& arg1, const DATA_EXPR2& arg2, RewriterCompilingJitty* tr)
        : m_head(head), m_arg0(arg0), m_arg1(arg1), m_arg2(arg2), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        make_application(local_store, [&](data_expression& r){ local_rewrite(r, m_head); }, [&](data_expression& r){ local_rewrite(r, m_arg0); }, [&](data_expression& r){ local_rewrite(r, m_arg1); }, [&](data_expression& r){ local_rewrite(r, m_arg2); });
        rewrite_with_arguments_in_normal_form(local_store, local_store, this_rewriter);
        return local_store;
      }

      void normal_form(data_expression& result) const
      {
        make_application(result, [&](data_expression& result){ local_rewrite(result, m_head); }, [&](data_expression& result){ local_rewrite(result, m_arg0); }, [&](data_expression& result){ local_rewrite(result, m_arg1); }, [&](data_expression& result){ local_rewrite(result, m_arg2); });
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
      }

  };

  template < class HEAD, class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3 >
  class delayed_application4
  {
    protected:
      const HEAD& m_head;
      const DATA_EXPR0& m_arg0;
      const DATA_EXPR1& m_arg1;
      const DATA_EXPR2& m_arg2;
      const DATA_EXPR3& m_arg3;
      RewriterCompilingJitty* this_rewriter;

    public:
      delayed_application4(const HEAD& head, const DATA_EXPR0& arg0, const DATA_EXPR1& arg1, const DATA_EXPR2& arg2, const DATA_EXPR3& arg3, RewriterCompilingJitty* tr)
        : m_head(head), m_arg0(arg0), m_arg1(arg1), m_arg2(arg2), m_arg3(arg3), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        make_application(local_store, [&](data_expression& r){ local_rewrite(r, m_head); }, [&](data_expression& r){ local_rewrite(r, m_arg0); }, [&](data_expression& r){ local_rewrite(r, m_arg1); }, [&](data_expression& r){ local_rewrite(r, m_arg2); }, [&](data_expression& r){ local_rewrite(r, m_arg3); });
        rewrite_with_arguments_in_normal_form(local_store, local_store, this_rewriter);
        return local_store;
      }

      void normal_form(data_expression& result) const
      {
        make_application(result, [&](data_expression& result){ local_rewrite(result, m_head); }, [&](data_expression& result){ local_rewrite(result, m_arg0); }, [&](data_expression& result){ local_rewrite(result, m_arg1); }, [&](data_expression& result){ local_rewrite(result, m_arg2); }, [&](data_expression& result){ local_rewrite(result, m_arg3); });
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
      }

  };

  template < class HEAD, class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4 >
  class delayed_application5
  {
    protected:
      const HEAD& m_head;
      const DATA_EXPR0& m_arg0;
      const DATA_EXPR1& m_arg1;
      const DATA_EXPR2& m_arg2;
      const DATA_EXPR3& m_arg3;
      const DATA_EXPR4& m_arg4;
      RewriterCompilingJitty* this_rewriter;

    public:
      delayed_application5(const HEAD& head, const DATA_EXPR0& arg0, const DATA_EXPR1& arg1, const DATA_EXPR2& arg2, const DATA_EXPR3& arg3, const DATA_EXPR4& arg4, RewriterCompilingJitty* tr)
        : m_head(head), m_arg0(arg0), m_arg1(arg1), m_arg2(arg2), m_arg3(arg3), m_arg4(arg4), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        make_application(local_store, [&](data_expression& r){ local_rewrite(r, m_head); }, [&](data_expression& r){ local_rewrite(r, m_arg0); }, [&](data_expression& r){ local_rewrite(r, m_arg1); }, [&](data_expression& r){ local_rewrite(r, m_arg2); }, [&](data_expression& r){ local_rewrite(r, m_arg3); }, [&](data_expression& r){ local_rewrite(r, m_arg4); });
        rewrite_with_arguments_in_normal_form(local_store, local_store, this_rewriter);
        return local_store;
      }

      void normal_form(data_expression& result) const
      {
        make_application(result, [&](data_expression& result){ local_rewrite(result, m_head); }, [&](data_expression& result){ local_rewrite(result, m_arg0); }, [&](data_expression& result){ local_rewrite(result, m_arg1); }, [&](data_expression& result){ local_rewrite(result, m_arg2); }, [&](data_expression& result){ local_rewrite(result, m_arg3); }, [&](data_expression& result){ local_rewrite(result, m_arg4); });
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
      }

  };

  // We're declaring static members in a struct rather than simple functions in
  // the global scope, so that we don't have to worry about forward declarations.
  // [29] C_Row1: Row1 # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_29_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_row, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dae0) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[29], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_29_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_29_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_29_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_29_2(result, t[0], t[1], this_rewriter); }


  // [29] C_Row1: Row1 # Piece -> Piece
  static inline void rewr_29_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063f90));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [28] pi_Row4: Row -> Piece
  template < class DATA_EXPR0>
  static inline void rewr_28_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, pi_Row4(@var_1)))))), @@F(row, @@N(@@N(@@N(@@N(@@S(@var_4, @@R(@var_4)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Row4(@var_1), pi_Row4(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_29_2(result, var_0, delayed_rewr_28_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, pi_Row4(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          const data_expression& var_4 = down_cast<data_expression>(arg0[5]); // S2
          result.assign(var_4, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_4
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_52_3(result, var_0, delayed_rewr_28_1<data_expression>(var_1, this_rewriter), delayed_rewr_28_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Row4(@var_1), pi_Row4(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[28], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_28_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_28_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_28_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_28_1(result, t[0], this_rewriter); }


  // [28] pi_Row4: Row -> Piece
  template < class DATA_EXPR0>
  class delayed_rewr_28_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_28_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_28_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_28_1(result, m_t0, this_rewriter);
      }
  };
  
  // [28] pi_Row4: Row -> Piece
  static inline void rewr_28_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063970));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [27] pi_Row3: Row -> Piece
  template < class DATA_EXPR0>
  static inline void rewr_27_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, pi_Row3(@var_1)))))), @@F(row, @@N(@@N(@@N(@@S(@var_3, @@N(@@R(@var_3)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Row3(@var_1), pi_Row3(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_29_2(result, var_0, delayed_rewr_27_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, pi_Row3(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          const data_expression& var_3 = down_cast<data_expression>(arg0[4]); // S2
          result.assign(var_3, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_3
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_52_3(result, var_0, delayed_rewr_27_1<data_expression>(var_1, this_rewriter), delayed_rewr_27_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Row3(@var_1), pi_Row3(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[27], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_27_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_27_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_27_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_27_1(result, t[0], this_rewriter); }


  // [27] pi_Row3: Row -> Piece
  template < class DATA_EXPR0>
  class delayed_rewr_27_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_27_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_27_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_27_1(result, m_t0, this_rewriter);
      }
  };
  
  // [27] pi_Row3: Row -> Piece
  static inline void rewr_27_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150601d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [26] pi_Row2: Row -> Piece
  template < class DATA_EXPR0>
  static inline void rewr_26_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, pi_Row2(@var_1)))))), @@F(row, @@N(@@N(@@S(@var_2, @@N(@@N(@@R(@var_2)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Row2(@var_1), pi_Row2(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_29_2(result, var_0, delayed_rewr_26_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, pi_Row2(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
          result.assign(var_2, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_2
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_52_3(result, var_0, delayed_rewr_26_1<data_expression>(var_1, this_rewriter), delayed_rewr_26_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Row2(@var_1), pi_Row2(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[26], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_26_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_26_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_26_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_26_1(result, t[0], this_rewriter); }


  // [26] pi_Row2: Row -> Piece
  template < class DATA_EXPR0>
  class delayed_rewr_26_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_26_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_26_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_26_1(result, m_t0, this_rewriter);
      }
  };
  
  // [26] pi_Row2: Row -> Piece
  static inline void rewr_26_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063940));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [25] pi_Row: Row -> Piece
  template < class DATA_EXPR0>
  static inline void rewr_25_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, pi_Row(@var_1)))))), @@F(row, @@N(@@S(@var_1, @@N(@@N(@@N(@@R(@var_1)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Row(@var_1), pi_Row(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_29_2(result, var_0, delayed_rewr_25_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, pi_Row(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
          result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_1
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_52_3(result, var_0, delayed_rewr_25_1<data_expression>(var_1, this_rewriter), delayed_rewr_25_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Row(@var_1), pi_Row(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[25], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_25_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_25_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_25_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_25_1(result, t[0], this_rewriter); }


  // [25] pi_Row: Row -> Piece
  template < class DATA_EXPR0>
  class delayed_rewr_25_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_25_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_25_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_25_1(result, m_t0, this_rewriter);
      }
  };
  
  // [25] pi_Row: Row -> Piece
  static inline void rewr_25_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150640a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [24] pi_Row1: Row -> Piece
  template < class DATA_EXPR0>
  static inline void rewr_24_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, pi_Row1(@var_1)))))), @@F(row, @@S(@var_0, @@N(@@N(@@N(@@N(@@R(@var_0)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Row1(@var_1), pi_Row1(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_29_2(result, var_0, delayed_rewr_24_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, pi_Row1(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
          result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_52_3(result, var_0, delayed_rewr_24_1<data_expression>(var_1, this_rewriter), delayed_rewr_24_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Row1(@var_1), pi_Row1(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[24], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_24_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_24_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_24_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_24_1(result, t[0], this_rewriter); }


  // [24] pi_Row1: Row -> Piece
  template < class DATA_EXPR0>
  class delayed_rewr_24_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_24_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_24_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_24_1(result, m_t0, this_rewriter);
      }
  };
  
  // [24] pi_Row1: Row -> Piece
  static inline void rewr_24_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063a60));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [23] C_Row1: Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_23_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_row, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dae0) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@F(c_row, @@R(@var_0), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address(arg1) == 0x55e114f5dae0) // F1
      {
        result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[23], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_23_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_23_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_23_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_23_2(result, t[0], t[1], this_rewriter); }


  // [23] C_Row1: Row1 # Row1 -> Row1
  static inline void rewr_23_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063a30));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [22] Det_Row1: Row -> Row1
  template < class DATA_EXPR0>
  static inline void rewr_22_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Row1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Row1(@var_0, Det_Row1(@var_1)))))), @@F(row, @@N(@@N(@@N(@@N(@@R(c_row))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, Det_Row1(@var_1), Det_Row1(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5de00) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_23_2(result, var_0, delayed_rewr_22_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Row1(@var_0, Det_Row1(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063a00));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 c_row
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e850) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_51_3(result, var_0, delayed_rewr_22_1<data_expression>(var_1, this_rewriter), delayed_rewr_22_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, Det_Row1(@var_1), Det_Row1(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[22], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_22_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_22_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_22_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_22_1(result, t[0], this_rewriter); }


  // [22] Det_Row1: Row -> Row1
  template < class DATA_EXPR0>
  class delayed_rewr_22_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_22_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_22_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_22_1(result, m_t0, this_rewriter);
      }
  };
  
  // [22] Det_Row1: Row -> Row1
  static inline void rewr_22_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150602c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [21] C_Row1: Row1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_21_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_row, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dae0) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[21], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_21_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_21_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_21_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_21_2(result, t[0], t[1], this_rewriter); }


  // [21] C_Row1: Row1 # Row -> Row
  static inline void rewr_21_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150603e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [20] C_Board1: Board1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_20_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_col, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dab8) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[20], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_20_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_20_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_20_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_20_2(result, t[0], t[1], this_rewriter); }


  // [20] C_Board1: Board1 # Row -> Row
  static inline void rewr_20_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150601a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [19] pi_Board4: Board -> Row
  template < class DATA_EXPR0>
  static inline void rewr_19_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, pi_Board4(@var_1)))))), @@F(col, @@N(@@N(@@N(@@N(@@S(@var_4, @@R(@var_4)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Board4(@var_1), pi_Board4(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_20_2(result, var_0, delayed_rewr_19_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, pi_Board4(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          const data_expression& var_4 = down_cast<data_expression>(arg0[5]); // S2
          result.assign(var_4, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_4
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_50_3(result, var_0, delayed_rewr_19_1<data_expression>(var_1, this_rewriter), delayed_rewr_19_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Board4(@var_1), pi_Board4(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[19], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_19_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_19_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_19_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_19_1(result, t[0], this_rewriter); }


  // [19] pi_Board4: Board -> Row
  template < class DATA_EXPR0>
  class delayed_rewr_19_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_19_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_19_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_19_1(result, m_t0, this_rewriter);
      }
  };
  
  // [19] pi_Board4: Board -> Row
  static inline void rewr_19_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063dc0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [18] pi_Board3: Board -> Row
  template < class DATA_EXPR0>
  static inline void rewr_18_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, pi_Board3(@var_1)))))), @@F(col, @@N(@@N(@@N(@@S(@var_3, @@N(@@R(@var_3)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Board3(@var_1), pi_Board3(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_20_2(result, var_0, delayed_rewr_18_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, pi_Board3(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          const data_expression& var_3 = down_cast<data_expression>(arg0[4]); // S2
          result.assign(var_3, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_3
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_50_3(result, var_0, delayed_rewr_18_1<data_expression>(var_1, this_rewriter), delayed_rewr_18_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Board3(@var_1), pi_Board3(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[18], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_18_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_18_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_18_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_18_1(result, t[0], this_rewriter); }


  // [18] pi_Board3: Board -> Row
  template < class DATA_EXPR0>
  class delayed_rewr_18_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_18_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_18_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_18_1(result, m_t0, this_rewriter);
      }
  };
  
  // [18] pi_Board3: Board -> Row
  static inline void rewr_18_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150603b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [17] pi_Board2: Board -> Row
  template < class DATA_EXPR0>
  static inline void rewr_17_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, pi_Board2(@var_1)))))), @@F(col, @@N(@@N(@@S(@var_2, @@N(@@N(@@R(@var_2)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Board2(@var_1), pi_Board2(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_20_2(result, var_0, delayed_rewr_17_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, pi_Board2(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
          result.assign(var_2, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_2
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_50_3(result, var_0, delayed_rewr_17_1<data_expression>(var_1, this_rewriter), delayed_rewr_17_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Board2(@var_1), pi_Board2(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[17], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_17_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_17_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_17_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_17_1(result, t[0], this_rewriter); }


  // [17] pi_Board2: Board -> Row
  template < class DATA_EXPR0>
  class delayed_rewr_17_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_17_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_17_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_17_1(result, m_t0, this_rewriter);
      }
  };
  
  // [17] pi_Board2: Board -> Row
  static inline void rewr_17_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060640));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [16] pi_Board: Board -> Row
  template < class DATA_EXPR0>
  static inline void rewr_16_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, pi_Board(@var_1)))))), @@F(col, @@N(@@S(@var_1, @@N(@@N(@@N(@@R(@var_1)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Board(@var_1), pi_Board(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_20_2(result, var_0, delayed_rewr_16_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, pi_Board(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
          result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_1
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_50_3(result, var_0, delayed_rewr_16_1<data_expression>(var_1, this_rewriter), delayed_rewr_16_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Board(@var_1), pi_Board(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[16], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_16_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_16_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_16_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_16_1(result, t[0], this_rewriter); }


  // [16] pi_Board: Board -> Row
  template < class DATA_EXPR0>
  class delayed_rewr_16_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_16_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_16_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_16_1(result, m_t0, this_rewriter);
      }
  };
  
  // [16] pi_Board: Board -> Row
  static inline void rewr_16_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150604d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [15] pi_Board1: Board -> Row
  template < class DATA_EXPR0>
  static inline void rewr_15_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, pi_Board1(@var_1)))))), @@F(col, @@S(@var_0, @@N(@@N(@@N(@@N(@@R(@var_0)))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, pi_Board1(@var_1), pi_Board1(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_20_2(result, var_0, delayed_rewr_15_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, pi_Board1(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
          result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_50_3(result, var_0, delayed_rewr_15_1<data_expression>(var_1, this_rewriter), delayed_rewr_15_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, pi_Board1(@var_1), pi_Board1(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[15], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_15_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_15_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_15_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_15_1(result, t[0], this_rewriter); }


  // [15] pi_Board1: Board -> Row
  template < class DATA_EXPR0>
  class delayed_rewr_15_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_15_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_15_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_15_1(result, m_t0, this_rewriter);
      }
  };
  
  // [15] pi_Board1: Board -> Row
  static inline void rewr_15_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060130));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [14] C_Board1: Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_14_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_col, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dab8) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@F(c_col, @@R(@var_0), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address(arg1) == 0x55e114f5dab8) // F1
      {
        result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[14], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_14_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_14_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_14_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_14_2(result, t[0], t[1], this_rewriter); }


  // [14] C_Board1: Board1 # Board1 -> Board1
  static inline void rewr_14_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150639a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [13] Det_Board1: Board -> Board1
  template < class DATA_EXPR0>
  static inline void rewr_13_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(C_Board1, @@S(@var_0, @@N(@@S(@var_1, @@R(C_Board1(@var_0, Det_Board1(@var_1)))))), @@F(col, @@N(@@N(@@N(@@N(@@R(c_col))))), @@F(if, @@S(@var_0, @@N(@@S(@var_1, @@N(@@S(@var_2, @@R(if(@var_0, Det_Board1(@var_1), Det_Board1(@var_2)))))))), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dc98) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
        rewr_14_2(result, var_0, delayed_rewr_13_1<data_expression>(var_1, this_rewriter), this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 C_Board1(@var_0, Det_Board1(@var_1))
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063ff0));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 c_col
        }
        else
        {
          if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e788) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
            const data_expression& var_1 = down_cast<data_expression>(arg0[2]); // S2
            const data_expression& var_2 = down_cast<data_expression>(arg0[3]); // S2
            rewr_48_3(result, var_0, delayed_rewr_13_1<data_expression>(var_1, this_rewriter), delayed_rewr_13_1<data_expression>(var_2, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 if(@var_0, Det_Board1(@var_1), Det_Board1(@var_2))
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[13], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_13_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_13_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_13_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_13_1(result, t[0], this_rewriter); }


  // [13] Det_Board1: Board -> Board1
  template < class DATA_EXPR0>
  class delayed_rewr_13_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_13_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_13_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_13_1(result, m_t0, this_rewriter);
      }
  };
  
  // [13] Det_Board1: Board -> Board1
  static inline void rewr_13_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150639d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [12] C_Board1: Board1 # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_12_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@N(@@S(@var_1, @@R(@var_1)))
    {
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      local_rewrite(result, var_1);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 @var_1
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(c_col, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X)
    {
      if (uint_address(arg0) == 0x55e114f5dab8) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[12], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_12_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_12_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_12_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_12_2(result, t[0], t[1], this_rewriter); }


  // [12] C_Board1: Board1 # Board -> Board
  static inline void rewr_12_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060770));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [9] finalBoard: Board
  static inline void rewr_9_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@R(col(row(white, white, white, white, white), row(black, white, white, white, white), row(black, black, empty, white, white), row(black, black, black, black, white), row(black, black, black, black, black)))
    {
      result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150606a0));
;
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R2a
    }
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150606a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [8] initialBoard: Board
  static inline void rewr_8_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@R(col(row(black, black, black, black, black), row(white, black, black, black, black), row(white, white, empty, black, black), row(white, white, white, white, black), row(white, white, white, white, white)))
    {
      result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060500));
;
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R2a
    }
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060500));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [7] setPiece: Pos # Piece # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_7_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@F(@cDub, @@F(true, @@D(@@N(@@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@F(row, @@S(@var_1, @@N(@@S(@var_2, @@N(@@N(@@S(@var_4, @@N(@@S(@var_5, @@R(row(@var_1, @var_2, @var_0, @var_4, @var_5)))))))))), @@X)))))), @@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@F(row, @@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@R(row(@var_1, @var_2, @var_3, @var_4, @var_0)))))))))), @@X))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@D(@@X))))), @@F(false, @@D(@@N(@@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@F(row, @@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@N(@@S(@var_5, @@R(row(@var_1, @var_2, @var_3, @var_0, @var_5)))))))))), @@X))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@F(row, @@S(@var_1, @@N(@@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(row(@var_1, @var_0, @var_3, @var_4, @var_5)))))))))), @@X)))))), @@D(@@X))))), @@D(@@X))), @@F(1, @@D(@@N(@@S(@var_0, @@N(@@F(row, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(row(@var_0, @var_2, @var_3, @var_4, @var_5)))))))))), @@X))))), @@X))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e1c0) // F1
      {
        if (uint_address(arg0[1]) == 0x55e114f5e170) // F2a true
        {
          const data_expression& t1 = down_cast<data_expression>(arg0[1]);
          if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
          {
            const data_expression& t2 = down_cast<data_expression>(arg0[2]);
            const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
            if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e3f0) // F1
            {
              const data_expression& var_1 = down_cast<data_expression>(arg2[1]); // S2
              const data_expression& var_2 = down_cast<data_expression>(arg2[2]); // S2
              const data_expression& var_4 = down_cast<data_expression>(arg2[4]); // S2
              const data_expression& var_5 = down_cast<data_expression>(arg2[5]); // S2
              rewr_41_5(result, var_1, var_2, var_0, var_4, var_5, this_rewriter);
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 row(@var_1, @var_2, @var_0, @var_4, @var_5)
            }
            else
            {
            }
          }
          else
          {
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e3f0) // F1
                  {
                    const data_expression& var_1 = down_cast<data_expression>(arg2[1]); // S2
                    const data_expression& var_2 = down_cast<data_expression>(arg2[2]); // S2
                    const data_expression& var_3 = down_cast<data_expression>(arg2[3]); // S2
                    const data_expression& var_4 = down_cast<data_expression>(arg2[4]); // S2
                    rewr_41_5(result, var_1, var_2, var_3, var_4, var_0, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 row(@var_1, @var_2, @var_3, @var_4, @var_0)
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
            }
          }
        }
        else
        {
          if (uint_address(arg0[1]) == 0x55e114f5e198) // F2a false
          {
            const data_expression& t1 = down_cast<data_expression>(arg0[1]);
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e3f0) // F1
                  {
                    const data_expression& var_1 = down_cast<data_expression>(arg2[1]); // S2
                    const data_expression& var_2 = down_cast<data_expression>(arg2[2]); // S2
                    const data_expression& var_3 = down_cast<data_expression>(arg2[3]); // S2
                    const data_expression& var_5 = down_cast<data_expression>(arg2[5]); // S2
                    rewr_41_5(result, var_1, var_2, var_3, var_0, var_5, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 row(@var_1, @var_2, @var_3, @var_0, @var_5)
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
              if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
              {
                const data_expression& t2 = down_cast<data_expression>(arg0[2]);
                const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e3f0) // F1
                {
                  const data_expression& var_1 = down_cast<data_expression>(arg2[1]); // S2
                  const data_expression& var_3 = down_cast<data_expression>(arg2[3]); // S2
                  const data_expression& var_4 = down_cast<data_expression>(arg2[4]); // S2
                  const data_expression& var_5 = down_cast<data_expression>(arg2[5]); // S2
                  rewr_41_5(result, var_1, var_0, var_3, var_4, var_5, this_rewriter);
                  this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                  return; // R1 row(@var_1, @var_0, @var_3, @var_4, @var_5)
                }
                else
                {
                }
              }
              else
              {
              }
            }
          }
          else
          {
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5df68) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e3f0) // F1
          {
            const data_expression& var_2 = down_cast<data_expression>(arg2[2]); // S2
            const data_expression& var_3 = down_cast<data_expression>(arg2[3]); // S2
            const data_expression& var_4 = down_cast<data_expression>(arg2[4]); // S2
            const data_expression& var_5 = down_cast<data_expression>(arg2[5]); // S2
            rewr_41_5(result, var_0, var_2, var_3, var_4, var_5, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 row(@var_0, @var_2, @var_3, @var_4, @var_5)
          }
          else
          {
          }
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[7], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); }, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_7_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_7_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_7_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_7_3(result, t[0], t[1], t[2], this_rewriter); }


  // [7] setPiece: Pos # Piece # Row -> Row
  static inline void rewr_7_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505cf00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [6] getPiece: Pos # Row -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_6_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@F(@cDub, @@F(true, @@D(@@N(@@F(1, @@D(@@D(@@N(@@F(row, @@N(@@N(@@S(@var_2, @@N(@@N(@@R(@var_2)))))), @@X)))), @@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@F(row, @@N(@@N(@@N(@@N(@@S(@var_4, @@R(@var_4)))))), @@X))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@D(@@X))))), @@F(false, @@D(@@N(@@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@F(row, @@N(@@N(@@N(@@S(@var_3, @@N(@@R(@var_3)))))), @@X))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@F(1, @@D(@@D(@@N(@@F(row, @@N(@@S(@var_1, @@N(@@N(@@N(@@R(@var_1)))))), @@X)))), @@D(@@X))))), @@D(@@X))), @@F(1, @@D(@@N(@@F(row, @@S(@var_0, @@N(@@N(@@N(@@N(@@R(@var_0)))))), @@X))), @@X))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e1c0) // F1
      {
        if (uint_address(arg0[1]) == 0x55e114f5e170) // F2a true
        {
          const data_expression& t1 = down_cast<data_expression>(arg0[1]);
          if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
          {
            const data_expression& t2 = down_cast<data_expression>(arg0[2]);
            if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
            {
              const data_expression& var_2 = down_cast<data_expression>(arg1[3]); // S2
              result.assign(var_2, *this_rewriter->m_thread_aterm_pool);
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 @var_2
            }
            else
            {
            }
          }
          else
          {
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
                  {
                    const data_expression& var_4 = down_cast<data_expression>(arg1[5]); // S2
                    result.assign(var_4, *this_rewriter->m_thread_aterm_pool);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 @var_4
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
            }
          }
        }
        else
        {
          if (uint_address(arg0[1]) == 0x55e114f5e198) // F2a false
          {
            const data_expression& t1 = down_cast<data_expression>(arg0[1]);
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
                  {
                    const data_expression& var_3 = down_cast<data_expression>(arg1[4]); // S2
                    result.assign(var_3, *this_rewriter->m_thread_aterm_pool);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 @var_3
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
              if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
              {
                const data_expression& t2 = down_cast<data_expression>(arg0[2]);
                if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
                {
                  const data_expression& var_1 = down_cast<data_expression>(arg1[2]); // S2
                  result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
                  this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                  return; // R1 @var_1
                }
                else
                {
                }
              }
              else
              {
              }
            }
          }
          else
          {
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5df68) // F1
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
          {
            const data_expression& var_0 = down_cast<data_expression>(arg1[1]); // S2
            result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_0
          }
          else
          {
          }
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[6], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_6_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_6_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_6_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_6_2(result, t[0], t[1], this_rewriter); }


  // [6] getPiece: Pos # Row -> Piece
  static inline void rewr_6_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060090));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [5] setPiece: Pos # Pos # Piece # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_5_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(3)
    data_expression& arg3(std::is_convertible<DATA_EXPR3, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf3))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR3, const data_expression&>::value)
    {
      local_rewrite(arg3, arg_not_nf3);
    }
    // Considering argument 3
    // @@F(@cDub, @@F(true, @@D(@@N(@@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@S(@var_1, @@N(@@F(col, @@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(col(@var_2, @var_3, setPiece(@var_0, @var_1, @var_4), @var_5, @var_6))))))))))), @@X)))))))), @@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@S(@var_1, @@N(@@F(col, @@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(col(@var_2, @var_3, @var_4, @var_5, setPiece(@var_0, @var_1, @var_6)))))))))))), @@X))))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@D(@@X))))), @@F(false, @@D(@@N(@@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@S(@var_1, @@N(@@F(col, @@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(col(@var_2, @var_3, @var_4, setPiece(@var_0, @var_1, @var_5), @var_6))))))))))), @@X))))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@S(@var_1, @@N(@@F(col, @@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(col(@var_2, setPiece(@var_0, @var_1, @var_3), @var_4, @var_5, @var_6))))))))))), @@X)))))))), @@D(@@X))))), @@D(@@X))), @@F(1, @@D(@@N(@@S(@var_0, @@N(@@S(@var_1, @@N(@@F(col, @@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(col(setPiece(@var_0, @var_1, @var_2), @var_3, @var_4, @var_5, @var_6))))))))))), @@X))))))), @@X))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e1c0) // F1
      {
        if (uint_address(arg0[1]) == 0x55e114f5e170) // F2a true
        {
          const data_expression& t1 = down_cast<data_expression>(arg0[1]);
          if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
          {
            const data_expression& t2 = down_cast<data_expression>(arg0[2]);
            const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
            const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
            if (uint_address((is_function_symbol(arg3) ? arg3 : arg3[0])) == 0x55e114f5e2b0) // F1
            {
              const data_expression& var_2 = down_cast<data_expression>(arg3[1]); // S2
              const data_expression& var_3 = down_cast<data_expression>(arg3[2]); // S2
              const data_expression& var_4 = down_cast<data_expression>(arg3[3]); // S2
              const data_expression& var_5 = down_cast<data_expression>(arg3[4]); // S2
              const data_expression& var_6 = down_cast<data_expression>(arg3[5]); // S2
              rewr_40_5(result, var_2, var_3, delayed_rewr_7_3<DATA_EXPR1, DATA_EXPR2, data_expression>(var_0, var_1, var_4, this_rewriter), var_5, var_6, this_rewriter);
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 col(@var_2, @var_3, setPiece(@var_0, @var_1, @var_4), @var_5, @var_6)
            }
            else
            {
            }
          }
          else
          {
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
                  if (uint_address((is_function_symbol(arg3) ? arg3 : arg3[0])) == 0x55e114f5e2b0) // F1
                  {
                    const data_expression& var_2 = down_cast<data_expression>(arg3[1]); // S2
                    const data_expression& var_3 = down_cast<data_expression>(arg3[2]); // S2
                    const data_expression& var_4 = down_cast<data_expression>(arg3[3]); // S2
                    const data_expression& var_5 = down_cast<data_expression>(arg3[4]); // S2
                    const data_expression& var_6 = down_cast<data_expression>(arg3[5]); // S2
                    rewr_40_5(result, var_2, var_3, var_4, var_5, delayed_rewr_7_3<DATA_EXPR1, DATA_EXPR2, data_expression>(var_0, var_1, var_6, this_rewriter), this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 col(@var_2, @var_3, @var_4, @var_5, setPiece(@var_0, @var_1, @var_6))
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
            }
          }
        }
        else
        {
          if (uint_address(arg0[1]) == 0x55e114f5e198) // F2a false
          {
            const data_expression& t1 = down_cast<data_expression>(arg0[1]);
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
                  if (uint_address((is_function_symbol(arg3) ? arg3 : arg3[0])) == 0x55e114f5e2b0) // F1
                  {
                    const data_expression& var_2 = down_cast<data_expression>(arg3[1]); // S2
                    const data_expression& var_3 = down_cast<data_expression>(arg3[2]); // S2
                    const data_expression& var_4 = down_cast<data_expression>(arg3[3]); // S2
                    const data_expression& var_5 = down_cast<data_expression>(arg3[4]); // S2
                    const data_expression& var_6 = down_cast<data_expression>(arg3[5]); // S2
                    rewr_40_5(result, var_2, var_3, var_4, delayed_rewr_7_3<DATA_EXPR1, DATA_EXPR2, data_expression>(var_0, var_1, var_5, this_rewriter), var_6, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 col(@var_2, @var_3, @var_4, setPiece(@var_0, @var_1, @var_5), @var_6)
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
              if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
              {
                const data_expression& t2 = down_cast<data_expression>(arg0[2]);
                const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
                if (uint_address((is_function_symbol(arg3) ? arg3 : arg3[0])) == 0x55e114f5e2b0) // F1
                {
                  const data_expression& var_2 = down_cast<data_expression>(arg3[1]); // S2
                  const data_expression& var_3 = down_cast<data_expression>(arg3[2]); // S2
                  const data_expression& var_4 = down_cast<data_expression>(arg3[3]); // S2
                  const data_expression& var_5 = down_cast<data_expression>(arg3[4]); // S2
                  const data_expression& var_6 = down_cast<data_expression>(arg3[5]); // S2
                  rewr_40_5(result, var_2, delayed_rewr_7_3<DATA_EXPR1, DATA_EXPR2, data_expression>(var_0, var_1, var_3, this_rewriter), var_4, var_5, var_6, this_rewriter);
                  this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                  return; // R1 col(@var_2, setPiece(@var_0, @var_1, @var_3), @var_4, @var_5, @var_6)
                }
                else
                {
                }
              }
              else
              {
              }
            }
          }
          else
          {
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5df68) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
          if (uint_address((is_function_symbol(arg3) ? arg3 : arg3[0])) == 0x55e114f5e2b0) // F1
          {
            const data_expression& var_2 = down_cast<data_expression>(arg3[1]); // S2
            const data_expression& var_3 = down_cast<data_expression>(arg3[2]); // S2
            const data_expression& var_4 = down_cast<data_expression>(arg3[3]); // S2
            const data_expression& var_5 = down_cast<data_expression>(arg3[4]); // S2
            const data_expression& var_6 = down_cast<data_expression>(arg3[5]); // S2
            rewr_40_5(result, delayed_rewr_7_3<DATA_EXPR1, DATA_EXPR2, data_expression>(var_0, var_1, var_2, this_rewriter), var_3, var_4, var_5, var_6, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 col(setPiece(@var_0, @var_1, @var_2), @var_3, @var_4, @var_5, @var_6)
          }
          else
          {
          }
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[5], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf2); }, arg3);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_5_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_5_4(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), this_rewriter); }

  static inline void rewr_5_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_5_4(result, t[0], t[1], t[2], t[3], this_rewriter); }


  // [7] setPiece: Pos # Piece # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  class delayed_rewr_7_3
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      const DATA_EXPR2& m_t2;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_7_3(const DATA_EXPR0& t0, const DATA_EXPR1& t1, const DATA_EXPR2& t2, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), m_t2(t2), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_7_3(local_store, m_t0, m_t1, m_t2, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_7_3(result, m_t0, m_t1, m_t2, this_rewriter);
      }
  };
  
  // [5] setPiece: Pos # Pos # Piece # Board -> Board
  static inline void rewr_5_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150607a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [4] getPiece: Pos # Pos # Board -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_4_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@F(@cDub, @@F(true, @@D(@@N(@@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@F(col, @@N(@@N(@@S(@var_3, @@N(@@N(@@R(getPiece(@var_0, @var_3))))))), @@X)))))), @@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@F(col, @@N(@@N(@@N(@@N(@@S(@var_5, @@R(getPiece(@var_0, @var_5))))))), @@X))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@D(@@X))))), @@F(false, @@D(@@N(@@F(@cDub, @@F(false, @@D(@@N(@@F(1, @@D(@@D(@@D(@@N(@@S(@var_0, @@N(@@F(col, @@N(@@N(@@N(@@S(@var_4, @@N(@@R(getPiece(@var_0, @var_4))))))), @@X))))))), @@D(@@D(@@X))))), @@D(@@D(@@X))), @@F(1, @@D(@@D(@@N(@@S(@var_0, @@N(@@F(col, @@N(@@S(@var_2, @@N(@@N(@@N(@@R(getPiece(@var_0, @var_2))))))), @@X)))))), @@D(@@X))))), @@D(@@X))), @@F(1, @@D(@@N(@@S(@var_0, @@N(@@F(col, @@S(@var_1, @@N(@@N(@@N(@@N(@@R(getPiece(@var_0, @var_1))))))), @@X))))), @@X))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e1c0) // F1
      {
        if (uint_address(arg0[1]) == 0x55e114f5e170) // F2a true
        {
          const data_expression& t1 = down_cast<data_expression>(arg0[1]);
          if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
          {
            const data_expression& t2 = down_cast<data_expression>(arg0[2]);
            const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
            if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e2b0) // F1
            {
              const data_expression& var_3 = down_cast<data_expression>(arg2[3]); // S2
              rewr_6_2(result, var_0, var_3, this_rewriter);
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 getPiece(@var_0, @var_3)
            }
            else
            {
            }
          }
          else
          {
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e2b0) // F1
                  {
                    const data_expression& var_5 = down_cast<data_expression>(arg2[5]); // S2
                    rewr_6_2(result, var_0, var_5, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 getPiece(@var_0, @var_5)
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
            }
          }
        }
        else
        {
          if (uint_address(arg0[1]) == 0x55e114f5e198) // F2a false
          {
            const data_expression& t1 = down_cast<data_expression>(arg0[1]);
            if (is_application_no_check(down_cast<data_expression>(arg0[2])) && uint_address(down_cast<data_expression>(arg0[2])[0]) == 0x55e114f5e1c0) // F2b @cDub
            {
              const data_expression& t2 = down_cast<data_expression>(arg0[2]);
              if (uint_address(t2[1]) == 0x55e114f5e198) // F2a false
              {
                const data_expression& t3 = down_cast<data_expression>(t2[1]);
                if (uint_address(t2[2]) == 0x55e114f5df68) // F2a @c1
                {
                  const data_expression& t4 = down_cast<data_expression>(t2[2]);
                  const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                  if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e2b0) // F1
                  {
                    const data_expression& var_4 = down_cast<data_expression>(arg2[4]); // S2
                    rewr_6_2(result, var_0, var_4, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 getPiece(@var_0, @var_4)
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
              else
              {
              }
            }
            else
            {
              if (uint_address(arg0[2]) == 0x55e114f5df68) // F2a @c1
              {
                const data_expression& t2 = down_cast<data_expression>(arg0[2]);
                const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
                if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e2b0) // F1
                {
                  const data_expression& var_2 = down_cast<data_expression>(arg2[2]); // S2
                  rewr_6_2(result, var_0, var_2, this_rewriter);
                  this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                  return; // R1 getPiece(@var_0, @var_2)
                }
                else
                {
                }
              }
              else
              {
              }
            }
          }
          else
          {
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5df68) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          if (uint_address((is_function_symbol(arg2) ? arg2 : arg2[0])) == 0x55e114f5e2b0) // F1
          {
            const data_expression& var_1 = down_cast<data_expression>(arg2[1]); // S2
            rewr_6_2(result, var_0, var_1, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 getPiece(@var_0, @var_1)
          }
          else
          {
          }
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[4], arg0, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); }, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_4_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_4_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_4_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_4_3(result, t[0], t[1], t[2], this_rewriter); }


  // [4] getPiece: Pos # Pos # Board -> Piece
  static inline void rewr_4_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150606d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [493] @equal_arguments: Board # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_493_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(col, @@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@D(@@N(@@M(@var_0, @@R(true), @@F(col, @@S(@var_6, @@N(@@S(@var_7, @@N(@@S(@var_8, @@N(@@S(@var_9, @@N(@@S(@var_10, @@R(@var_1 == @var_6 && @var_2 == @var_7 && @var_3 == @var_8 && @var_4 == @var_9 && @var_5 == @var_10)))))))))), @@X))))))))))))), @@N(@@M(@var_0, @@R(true), @@X))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
        const data_expression& var_3 = down_cast<data_expression>(arg0[3]); // S2
        const data_expression& var_4 = down_cast<data_expression>(arg0[4]); // S2
        const data_expression& var_5 = down_cast<data_expression>(arg0[5]); // S2
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e2b0) // F1
          {
            const data_expression& var_6 = down_cast<data_expression>(arg1[1]); // S2
            const data_expression& var_7 = down_cast<data_expression>(arg1[2]); // S2
            const data_expression& var_8 = down_cast<data_expression>(arg1[3]); // S2
            const data_expression& var_9 = down_cast<data_expression>(arg1[4]); // S2
            const data_expression& var_10 = down_cast<data_expression>(arg1[5]); // S2
            rewr_46_2(result, delayed_rewr_176_2<data_expression, data_expression>(var_1, var_6, this_rewriter), delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_176_2<data_expression, data_expression>>>>(delayed_rewr_176_2<data_expression, data_expression>(var_2, var_7, this_rewriter), delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_176_2<data_expression, data_expression>>>(delayed_rewr_176_2<data_expression, data_expression>(var_3, var_8, this_rewriter), delayed_rewr_46_2<delayed_rewr_176_2<data_expression, data_expression>, delayed_rewr_176_2<data_expression, data_expression>>(delayed_rewr_176_2<data_expression, data_expression>(var_4, var_9, this_rewriter), delayed_rewr_176_2<data_expression, data_expression>(var_5, var_10, this_rewriter), this_rewriter), this_rewriter), this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_1 == @var_6 && @var_2 == @var_7 && @var_3 == @var_8 && @var_4 == @var_9 && @var_5 == @var_10
          }
          else
          {
          }
        }
      }
      else
      {
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[493], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_493_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_493_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_493_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_493_2(result, t[0], t[1], this_rewriter); }


  // [46] &&: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_46_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_46_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_46_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_46_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [176] ==: Row # Row -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_176_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_176_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_176_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_176_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [493] @equal_arguments: Board # Board -> Bool
  static inline void rewr_493_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063f60));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [492] @to_pos: Board -> Pos
  template < class DATA_EXPR0>
  static inline void rewr_492_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(col, @@N(@@N(@@N(@@N(@@R(1))))), @@X)
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e2b0) // F1
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081520));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[492], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_492_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_492_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_492_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_492_1(result, t[0], this_rewriter); }


  // [492] @to_pos: Board -> Pos
  static inline void rewr_492_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11507fc70));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [478] @equal_arguments: Row # Row -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_478_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(row, @@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@D(@@N(@@M(@var_0, @@R(true), @@F(row, @@S(@var_6, @@N(@@S(@var_7, @@N(@@S(@var_8, @@N(@@S(@var_9, @@N(@@S(@var_10, @@R(@var_1 == @var_6 && @var_2 == @var_7 && @var_3 == @var_8 && @var_4 == @var_9 && @var_5 == @var_10)))))))))), @@X))))))))))))), @@N(@@M(@var_0, @@R(true), @@X))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
        const data_expression& var_3 = down_cast<data_expression>(arg0[3]); // S2
        const data_expression& var_4 = down_cast<data_expression>(arg0[4]); // S2
        const data_expression& var_5 = down_cast<data_expression>(arg0[5]); // S2
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e3f0) // F1
          {
            const data_expression& var_6 = down_cast<data_expression>(arg1[1]); // S2
            const data_expression& var_7 = down_cast<data_expression>(arg1[2]); // S2
            const data_expression& var_8 = down_cast<data_expression>(arg1[3]); // S2
            const data_expression& var_9 = down_cast<data_expression>(arg1[4]); // S2
            const data_expression& var_10 = down_cast<data_expression>(arg1[5]); // S2
            rewr_46_2(result, delayed_rewr_59_2<data_expression, data_expression>(var_1, var_6, this_rewriter), delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_59_2<data_expression, data_expression>>>>(delayed_rewr_59_2<data_expression, data_expression>(var_2, var_7, this_rewriter), delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_59_2<data_expression, data_expression>>>(delayed_rewr_59_2<data_expression, data_expression>(var_3, var_8, this_rewriter), delayed_rewr_46_2<delayed_rewr_59_2<data_expression, data_expression>, delayed_rewr_59_2<data_expression, data_expression>>(delayed_rewr_59_2<data_expression, data_expression>(var_4, var_9, this_rewriter), delayed_rewr_59_2<data_expression, data_expression>(var_5, var_10, this_rewriter), this_rewriter), this_rewriter), this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_1 == @var_6 && @var_2 == @var_7 && @var_3 == @var_8 && @var_4 == @var_9 && @var_5 == @var_10
          }
          else
          {
          }
        }
      }
      else
      {
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[478], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_478_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_478_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_478_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_478_2(result, t[0], t[1], this_rewriter); }


  // [59] ==: Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_59_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_59_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_59_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_59_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [478] @equal_arguments: Row # Row -> Bool
  static inline void rewr_478_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063e50));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [477] @to_pos: Row -> Pos
  template < class DATA_EXPR0>
  static inline void rewr_477_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(row, @@N(@@N(@@N(@@N(@@R(1))))), @@X)
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e3f0) // F1
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081520));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[477], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_477_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_477_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_477_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_477_1(result, t[0], this_rewriter); }


  // [477] @to_pos: Row -> Pos
  static inline void rewr_477_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060b40));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [463] @equal_arguments: Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_463_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(empty, @@D(@@N(@@M(@var_0, @@R(true), @@F(empty, @@R(true), @@X)))), @@F(black, @@D(@@N(@@M(@var_0, @@R(true), @@F(black, @@R(true), @@X)))), @@F(white, @@D(@@N(@@M(@var_0, @@R(true), @@F(white, @@R(true), @@X)))), @@N(@@M(@var_0, @@R(true), @@X))))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address(arg0) == 0x55e114f5e530) // F1
      {
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address(arg1) == 0x55e114f5e530) // F1
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e508) // F1
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
            if (uint_address(arg1) == 0x55e114f5e508) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 true
            }
            else
            {
            }
          }
        }
        else
        {
          if (uint_address(arg0) == 0x55e114f5e4e0) // F1
          {
            if (var_0 == arg1) // M
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 true
            }
            else
            {
              if (uint_address(arg1) == 0x55e114f5e4e0) // F1
              {
                result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
                this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                return; // R1 true
              }
              else
              {
              }
            }
          }
          else
          {
            if (var_0 == arg1) // M
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 true
            }
            else
            {
            }
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[463], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_463_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_463_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_463_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_463_2(result, t[0], t[1], this_rewriter); }


  // [463] @equal_arguments: Piece # Piece -> Bool
  static inline void rewr_463_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060670));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [462] @to_pos: Piece -> Pos
  template < class DATA_EXPR0>
  static inline void rewr_462_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(empty, @@R(3), @@F(black, @@R(1), @@F(white, @@R(2), @@X)))
    {
      if (uint_address(arg0) == 0x55e114f5e530) // F1
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060290));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 3
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e508) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081520));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 1
        }
        else
        {
          if (uint_address(arg0) == 0x55e114f5e4e0) // F1
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115043800));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 2
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[462], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_462_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_462_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_462_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_462_1(result, t[0], this_rewriter); }


  // [462] @to_pos: Piece -> Pos
  static inline void rewr_462_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060410));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [446] if: Bool # (Bool -> Bool) # (Bool -> Bool) -> Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_446_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[446], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_446_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_446_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_446_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_446_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [446] if: Bool # (Bool -> Bool) # (Bool -> Bool) -> Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_446_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[446], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_446_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_446_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_446_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_446_3(result, t[0], t[1], t[2], this_rewriter); }


  // [446] if: Bool # (Bool -> Bool) # (Bool -> Bool) -> Bool -> Bool
  static inline void rewr_446_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11507fce0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [445] !=: (Bool -> Bool) # (Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_445_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_444_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[445], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_445_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_445_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_445_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_445_2(result, t[0], t[1], this_rewriter); }


  // [444] ==: (Bool -> Bool) # (Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_444_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_444_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_444_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_444_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [445] !=: (Bool -> Bool) # (Bool -> Bool) -> Bool
  static inline void rewr_445_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115060700));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [444] ==: (Bool -> Bool) # (Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_444_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(0), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[444], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_444_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_444_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_444_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_444_2(result, t[0], t[1], this_rewriter); }


  // [444] ==: (Bool -> Bool) # (Bool -> Bool) -> Bool
  static inline void rewr_444_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150813a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [435] if: Bool # (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool # Piece # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_435_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[435], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_435_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_435_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_435_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_435_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [435] if: Bool # (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool # Piece # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_435_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[435], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_435_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_435_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_435_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_435_3(result, t[0], t[1], t[2], this_rewriter); }


  // [435] if: Bool # (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool # Piece # Piece -> Piece
  static inline void rewr_435_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114f10f00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [434] !=: (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_434_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_433_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[434], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_434_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_434_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_434_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_434_2(result, t[0], t[1], this_rewriter); }


  // [433] ==: (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_433_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_433_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_433_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_433_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [434] !=: (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool
  static inline void rewr_434_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081430));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [433] ==: (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_433_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x2,x5: Piece. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_59_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(2)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(2)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(1), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x2,x5: Piece. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[433], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_433_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_433_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_433_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_433_2(result, t[0], t[1], this_rewriter); }


  // [433] ==: (Bool # Piece # Piece -> Piece) # (Bool # Piece # Piece -> Piece) -> Bool
  static inline void rewr_433_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505cf30));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [428] if: Bool # (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool # Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_428_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[428], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_428_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_428_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_428_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_428_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [428] if: Bool # (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool # Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_428_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[428], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_428_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_428_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_428_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_428_3(result, t[0], t[1], t[2], this_rewriter); }


  // [428] if: Bool # (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool # Row1 # Row1 -> Row1
  static inline void rewr_428_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085910));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [427] !=: (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_427_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_426_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[427], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_427_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_427_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_427_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_427_2(result, t[0], t[1], this_rewriter); }


  // [426] ==: (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_426_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_426_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_426_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_426_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [427] !=: (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool
  static inline void rewr_427_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505ceb0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [426] ==: (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_426_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x2,x5: Row1. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_166_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(3)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(4)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(3)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(4)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(2), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x2,x5: Row1. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[426], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_426_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_426_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_426_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_426_2(result, t[0], t[1], this_rewriter); }


  // [426] ==: (Bool # Row1 # Row1 -> Row1) # (Bool # Row1 # Row1 -> Row1) -> Bool
  static inline void rewr_426_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505d270));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [421] if: Bool # (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool # Row # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_421_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[421], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_421_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_421_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_421_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_421_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [421] if: Bool # (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool # Row # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_421_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[421], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_421_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_421_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_421_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_421_3(result, t[0], t[1], t[2], this_rewriter); }


  // [421] if: Bool # (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool # Row # Row -> Row
  static inline void rewr_421_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150813d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [420] !=: (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_420_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_419_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[420], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_420_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_420_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_420_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_420_2(result, t[0], t[1], this_rewriter); }


  // [419] ==: (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_419_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_419_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_419_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_419_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [420] !=: (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool
  static inline void rewr_420_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa57a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [419] ==: (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_419_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x2,x5: Row. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(5)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(6)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(5)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(6)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(3), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x2,x5: Row. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[419], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_419_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_419_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_419_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_419_2(result, t[0], t[1], this_rewriter); }


  // [419] ==: (Bool # Row # Row -> Row) # (Bool # Row # Row -> Row) -> Bool
  static inline void rewr_419_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085820));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [414] if: Bool # (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool # Board # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_414_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[414], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_414_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_414_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_414_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_414_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [414] if: Bool # (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool # Board # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_414_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[414], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_414_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_414_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_414_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_414_3(result, t[0], t[1], t[2], this_rewriter); }


  // [414] if: Bool # (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool # Board # Board -> Board
  static inline void rewr_414_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa5770));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [413] !=: (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_413_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_412_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[413], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_413_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_413_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_413_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_413_2(result, t[0], t[1], this_rewriter); }


  // [412] ==: (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_412_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_412_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_412_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_412_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [413] !=: (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool
  static inline void rewr_413_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505d240));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [412] ==: (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_412_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x2,x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_182_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(7)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(7)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(4), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x2,x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[412], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_412_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_412_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_412_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_412_2(result, t[0], t[1], this_rewriter); }


  // [412] ==: (Bool # Board # Board -> Board) # (Bool # Board # Board -> Board) -> Bool
  static inline void rewr_412_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa49e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [407] if: Bool # (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool # Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_407_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[407], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_407_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_407_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_407_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_407_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [407] if: Bool # (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool # Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_407_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[407], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_407_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_407_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_407_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_407_3(result, t[0], t[1], t[2], this_rewriter); }


  // [407] if: Bool # (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool # Board1 # Board1 -> Board1
  static inline void rewr_407_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081c20));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [406] !=: (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_406_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_405_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[406], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_406_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_406_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_406_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_406_2(result, t[0], t[1], this_rewriter); }


  // [405] ==: (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_405_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_405_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_405_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_405_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [406] !=: (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool
  static inline void rewr_406_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063b30));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [405] ==: (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_405_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x2,x5: Board1. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_160_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(9)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(10)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(9)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(10)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(5), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x2,x5: Board1. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[405], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_405_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_405_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_405_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_405_2(result, t[0], t[1], this_rewriter); }


  // [405] ==: (Bool # Board1 # Board1 -> Board1) # (Bool # Board1 # Board1 -> Board1) -> Bool
  static inline void rewr_405_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa59c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [400] if: Bool # (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_400_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[400], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_400_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_400_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_400_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_400_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [400] if: Bool # (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_400_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[400], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_400_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_400_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_400_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_400_3(result, t[0], t[1], t[2], this_rewriter); }


  // [400] if: Bool # (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool # Bool -> Bool
  static inline void rewr_400_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa58b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [399] !=: (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_399_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_398_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[399], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_399_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_399_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_399_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_399_2(result, t[0], t[1], this_rewriter); }


  // [398] ==: (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_398_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_398_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_398_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_398_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [399] !=: (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool
  static inline void rewr_399_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11509db40));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [398] ==: (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_398_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Bool. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(11)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(11)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(6), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Bool. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[398], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_398_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_398_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_398_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_398_2(result, t[0], t[1], this_rewriter); }


  // [398] ==: (Bool # Bool -> Bool) # (Bool # Bool -> Bool) -> Bool
  static inline void rewr_398_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505d210));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [393] if: Bool # (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_393_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[393], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_393_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_393_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_393_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_393_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [393] if: Bool # (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_393_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[393], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_393_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_393_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_393_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_393_3(result, t[0], t[1], t[2], this_rewriter); }


  // [393] if: Bool # (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Piece # Piece -> Bool
  static inline void rewr_393_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa5880));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [392] !=: (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_392_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_391_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[392], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_392_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_392_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_392_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_392_2(result, t[0], t[1], this_rewriter); }


  // [391] ==: (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_391_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_391_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_391_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_391_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [392] !=: (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Bool
  static inline void rewr_392_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11509dbb0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [391] ==: (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_391_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Piece. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(12)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(12)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(7), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Piece. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[391], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_391_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_391_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_391_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_391_2(result, t[0], t[1], this_rewriter); }


  // [391] ==: (Piece # Piece -> Bool) # (Piece # Piece -> Bool) -> Bool
  static inline void rewr_391_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa4ae0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [386] if: Bool # (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Piece # Piece # Piece # Piece # Piece -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5, class DATA_EXPR6, class DATA_EXPR7>
  static inline void rewr_386_8(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, const DATA_EXPR6& arg_not_nf6, const DATA_EXPR7& arg_not_nf7, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_1(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_0(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5, @var_6)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
          const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4, @var_5, @var_6)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_1(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5, @var_6)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[386], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf6); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf7); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_386_8_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_386_8(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), term_not_in_normal_form(t[4], this_rewriter), this_rewriter); }

  static inline void rewr_386_8_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_386_8(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], t[3], t[4], this_rewriter); }


  // [386] if: Bool # (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Piece # Piece # Piece # Piece # Piece -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_386_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[386], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_386_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_386_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_386_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_386_3(result, t[0], t[1], t[2], this_rewriter); }


  // [386] if: Bool # (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Piece # Piece # Piece # Piece # Piece -> Row
  static inline void rewr_386_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115083860));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [385] !=: (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_385_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_384_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[385], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_385_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_385_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_385_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_385_2(result, t[0], t[1], this_rewriter); }


  // [384] ==: (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_384_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_384_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_384_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_384_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [385] !=: (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Bool
  static inline void rewr_385_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11509da10));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [384] ==: (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_384_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2,x3,x4,x9: Piece. @var_0(x1, x2, x3, x4, x9) == @var_1(x1, x2, x3, x4, x9)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application5<data_expression,data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(12)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(14)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(15)), this_rewriter), delayed_application5<data_expression,data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(12)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(14)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(15)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(8), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2,x3,x4,x9: Piece. @var_0(x1, x2, x3, x4, x9) == @var_1(x1, x2, x3, x4, x9)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[384], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_384_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_384_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_384_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_384_2(result, t[0], t[1], this_rewriter); }


  // [384] ==: (Piece # Piece # Piece # Piece # Piece -> Row) # (Piece # Piece # Piece # Piece # Piece -> Row) -> Bool
  static inline void rewr_384_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa4ab0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [379] if: Bool # (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Row # Row # Row # Row # Row -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5, class DATA_EXPR6, class DATA_EXPR7>
  static inline void rewr_379_8(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, const DATA_EXPR6& arg_not_nf6, const DATA_EXPR7& arg_not_nf7, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_1(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_0(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5, @var_6)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
          const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4, @var_5, @var_6)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@N(@@S(@var_6, @@R(@var_1(@var_2, @var_3, @var_4, @var_5, @var_6)))))))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        const DATA_EXPR7& var_6 = arg_not_nf7; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5), local_rewrite(var_6));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5, @var_6)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[379], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf6); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf7); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_379_8_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_379_8(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), term_not_in_normal_form(t[4], this_rewriter), this_rewriter); }

  static inline void rewr_379_8_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_379_8(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], t[3], t[4], this_rewriter); }


  // [379] if: Bool # (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Row # Row # Row # Row # Row -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_379_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[379], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_379_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_379_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_379_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_379_3(result, t[0], t[1], t[2], this_rewriter); }


  // [379] if: Bool # (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Row # Row # Row # Row # Row -> Board
  static inline void rewr_379_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150858c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [378] !=: (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_378_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_377_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[378], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_378_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_378_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_378_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_378_2(result, t[0], t[1], this_rewriter); }


  // [377] ==: (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_377_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_377_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_377_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_377_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [378] !=: (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Bool
  static inline void rewr_378_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa4a80));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [377] ==: (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_377_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2,x3,x4,x9: Row. @var_0(x1, x2, x3, x4, x9) == @var_1(x1, x2, x3, x4, x9)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_182_2(result, delayed_application5<data_expression,data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(5)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(18)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(19)), this_rewriter), delayed_application5<data_expression,data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(5)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(18)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(19)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(9), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2,x3,x4,x9: Row. @var_0(x1, x2, x3, x4, x9) == @var_1(x1, x2, x3, x4, x9)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[377], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_377_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_377_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_377_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_377_2(result, t[0], t[1], this_rewriter); }


  // [377] ==: (Row # Row # Row # Row # Row -> Board) # (Row # Row # Row # Row # Row -> Board) -> Bool
  static inline void rewr_377_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150602f0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [372] if: Bool # (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool # Pos -> Pos
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_372_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[372], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_372_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_372_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_372_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_372_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [372] if: Bool # (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool # Pos -> Pos
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_372_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[372], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_372_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_372_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_372_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_372_3(result, t[0], t[1], t[2], this_rewriter); }


  // [372] if: Bool # (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool # Pos -> Pos
  static inline void rewr_372_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150838e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [371] !=: (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_371_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_370_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[371], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_371_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_371_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_371_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_371_2(result, t[0], t[1], this_rewriter); }


  // [370] ==: (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_370_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_370_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_370_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_370_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [371] !=: (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool
  static inline void rewr_371_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2b20));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [370] ==: (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_370_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Bool, x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_60_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(10), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Bool, x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[370], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_370_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_370_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_370_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_370_2(result, t[0], t[1], this_rewriter); }


  // [370] ==: (Bool # Pos -> Pos) # (Bool # Pos -> Pos) -> Bool
  static inline void rewr_370_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505d1e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [365] if: Bool # (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool # Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_365_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[365], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_365_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_365_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_365_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_365_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [365] if: Bool # (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool # Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_365_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[365], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_365_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_365_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_365_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_365_3(result, t[0], t[1], t[2], this_rewriter); }


  // [365] if: Bool # (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool # Bool # Bool -> Bool
  static inline void rewr_365_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150809f0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [364] !=: (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_364_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_363_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[364], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_364_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_364_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_364_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_364_2(result, t[0], t[1], this_rewriter); }


  // [363] ==: (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_363_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_363_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_363_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_363_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [364] !=: (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool
  static inline void rewr_364_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115083800));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [363] ==: (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_363_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2,x5: Bool. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(21)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(22)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(0)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(21)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(22)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(11), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2,x5: Bool. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[363], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_363_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_363_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_363_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_363_2(result, t[0], t[1], this_rewriter); }


  // [363] ==: (Bool # Bool # Bool -> Bool) # (Bool # Bool # Bool -> Bool) -> Bool
  static inline void rewr_363_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2ac0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [358] if: Bool # (Int # Int -> Bool) # (Int # Int -> Bool) -> Int # Int -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_358_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[358], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_358_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_358_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_358_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_358_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [358] if: Bool # (Int # Int -> Bool) # (Int # Int -> Bool) -> Int # Int -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_358_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[358], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_358_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_358_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_358_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_358_3(result, t[0], t[1], t[2], this_rewriter); }


  // [358] if: Bool # (Int # Int -> Bool) # (Int # Int -> Bool) -> Int # Int -> Bool
  static inline void rewr_358_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085970));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [357] !=: (Int # Int -> Bool) # (Int # Int -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_357_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_356_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[357], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_357_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_357_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_357_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_357_2(result, t[0], t[1], this_rewriter); }


  // [356] ==: (Int # Int -> Bool) # (Int # Int -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_356_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_356_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_356_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_356_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [357] !=: (Int # Int -> Bool) # (Int # Int -> Bool) -> Bool
  static inline void rewr_357_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115083830));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [356] ==: (Int # Int -> Bool) # (Int # Int -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_356_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Int. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(23)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(24)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(23)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(24)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(12), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Int. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[356], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_356_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_356_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_356_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_356_2(result, t[0], t[1], this_rewriter); }


  // [356] ==: (Int # Int -> Bool) # (Int # Int -> Bool) -> Bool
  static inline void rewr_356_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa4950));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [351] if: Bool # (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Pos # Pos -> Int
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_351_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[351], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_351_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_351_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_351_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_351_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [351] if: Bool # (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Pos # Pos -> Int
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_351_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[351], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_351_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_351_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_351_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_351_3(result, t[0], t[1], t[2], this_rewriter); }


  // [351] if: Bool # (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Pos # Pos -> Int
  static inline void rewr_351_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa1020));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [350] !=: (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_350_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_349_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[350], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_350_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_350_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_350_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_350_2(result, t[0], t[1], this_rewriter); }


  // [349] ==: (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_349_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_349_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_349_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_349_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [350] !=: (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Bool
  static inline void rewr_350_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084d30));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [349] ==: (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_349_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_34_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(13), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[349], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_349_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_349_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_349_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_349_2(result, t[0], t[1], this_rewriter); }


  // [349] ==: (Pos # Pos -> Int) # (Pos # Pos -> Int) -> Bool
  static inline void rewr_349_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150855b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [344] if: Bool # (Nat -> Int) # (Nat -> Int) -> Nat -> Int
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_344_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[344], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_344_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_344_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_344_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_344_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [344] if: Bool # (Nat -> Int) # (Nat -> Int) -> Nat -> Int
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_344_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[344], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_344_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_344_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_344_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_344_3(result, t[0], t[1], t[2], this_rewriter); }


  // [344] if: Bool # (Nat -> Int) # (Nat -> Int) -> Nat -> Int
  static inline void rewr_344_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084d90));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [343] !=: (Nat -> Int) # (Nat -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_343_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_342_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[343], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_343_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_343_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_343_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_343_2(result, t[0], t[1], this_rewriter); }


  // [342] ==: (Nat -> Int) # (Nat -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_342_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_342_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_342_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_342_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [343] !=: (Nat -> Int) # (Nat -> Int) -> Bool
  static inline void rewr_343_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085090));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [342] ==: (Nat -> Int) # (Nat -> Int) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_342_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Nat. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_34_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(26)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(26)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(14), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Nat. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[342], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_342_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_342_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_342_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_342_2(result, t[0], t[1], this_rewriter); }


  // [342] ==: (Nat -> Int) # (Nat -> Int) -> Bool
  static inline void rewr_342_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115082700));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [333] if: Bool # (Pos -> Nat) # (Pos -> Nat) -> Pos -> Nat
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_333_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[333], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_333_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_333_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_333_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_333_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [333] if: Bool # (Pos -> Nat) # (Pos -> Nat) -> Pos -> Nat
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_333_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[333], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_333_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_333_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_333_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_333_3(result, t[0], t[1], t[2], this_rewriter); }


  // [333] if: Bool # (Pos -> Nat) # (Pos -> Nat) -> Pos -> Nat
  static inline void rewr_333_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150850c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [332] !=: (Pos -> Nat) # (Pos -> Nat) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_332_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_331_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[332], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_332_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_332_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_332_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_332_2(result, t[0], t[1], this_rewriter); }


  // [331] ==: (Pos -> Nat) # (Pos -> Nat) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_331_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_331_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_331_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_331_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [332] !=: (Pos -> Nat) # (Pos -> Nat) -> Bool
  static inline void rewr_332_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2af0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [331] ==: (Pos -> Nat) # (Pos -> Nat) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_331_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Pos. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_97_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(15), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Pos. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[331], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_331_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_331_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_331_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_331_2(result, t[0], t[1], this_rewriter); }


  // [331] ==: (Pos -> Nat) # (Pos -> Nat) -> Bool
  static inline void rewr_331_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150855e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [322] if: Bool # (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Row1 # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_322_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[322], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_322_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_322_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_322_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_322_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [322] if: Bool # (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Row1 # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_322_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[322], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_322_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_322_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_322_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_322_3(result, t[0], t[1], t[2], this_rewriter); }


  // [322] if: Bool # (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Row1 # Piece -> Piece
  static inline void rewr_322_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085700));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [321] !=: (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_321_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_320_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[321], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_321_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_321_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_321_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_321_2(result, t[0], t[1], this_rewriter); }


  // [320] ==: (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_320_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_320_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_320_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_320_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [321] !=: (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Bool
  static inline void rewr_321_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150850f0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [320] ==: (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_320_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Row1, x3: Piece. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_59_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(16), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Row1, x3: Piece. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[320], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_320_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_320_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_320_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_320_2(result, t[0], t[1], this_rewriter); }


  // [320] ==: (Row1 # Piece -> Piece) # (Row1 # Piece -> Piece) -> Bool
  static inline void rewr_320_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069750));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [315] if: Bool # (Row -> Piece) # (Row -> Piece) -> Row -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_315_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[315], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_315_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_315_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_315_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_315_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [315] if: Bool # (Row -> Piece) # (Row -> Piece) -> Row -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_315_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[315], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_315_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_315_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_315_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_315_3(result, t[0], t[1], t[2], this_rewriter); }


  // [315] if: Bool # (Row -> Piece) # (Row -> Piece) -> Row -> Piece
  static inline void rewr_315_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069f00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [314] !=: (Row -> Piece) # (Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_314_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_313_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[314], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_314_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_314_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_314_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_314_2(result, t[0], t[1], this_rewriter); }


  // [313] ==: (Row -> Piece) # (Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_313_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_313_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_313_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_313_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [314] !=: (Row -> Piece) # (Row -> Piece) -> Bool
  static inline void rewr_314_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069680));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [313] ==: (Row -> Piece) # (Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_313_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Row. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_59_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(17), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Row. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[313], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_313_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_313_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_313_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_313_2(result, t[0], t[1], this_rewriter); }


  // [313] ==: (Row -> Piece) # (Row -> Piece) -> Bool
  static inline void rewr_313_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2de0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [304] if: Bool # (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_304_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[304], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_304_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_304_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_304_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_304_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [304] if: Bool # (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_304_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[304], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_304_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_304_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_304_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_304_3(result, t[0], t[1], t[2], this_rewriter); }


  // [304] if: Bool # (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Row1 # Row1 -> Row1
  static inline void rewr_304_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115082730));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [303] !=: (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_303_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_302_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[303], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_303_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_303_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_303_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_303_2(result, t[0], t[1], this_rewriter); }


  // [302] ==: (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_302_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_302_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_302_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_302_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [303] !=: (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Bool
  static inline void rewr_303_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069ed0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [302] ==: (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_302_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Row1. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_166_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(28)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(28)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(18), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Row1. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[302], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_302_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_302_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_302_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_302_2(result, t[0], t[1], this_rewriter); }


  // [302] ==: (Row1 # Row1 -> Row1) # (Row1 # Row1 -> Row1) -> Bool
  static inline void rewr_302_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150696b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [297] if: Bool # (Row -> Row1) # (Row -> Row1) -> Row -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_297_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[297], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_297_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_297_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_297_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_297_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [297] if: Bool # (Row -> Row1) # (Row -> Row1) -> Row -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_297_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[297], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_297_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_297_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_297_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_297_3(result, t[0], t[1], t[2], this_rewriter); }


  // [297] if: Bool # (Row -> Row1) # (Row -> Row1) -> Row -> Row1
  static inline void rewr_297_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069780));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [296] !=: (Row -> Row1) # (Row -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_296_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_295_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[296], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_296_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_296_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_296_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_296_2(result, t[0], t[1], this_rewriter); }


  // [295] ==: (Row -> Row1) # (Row -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_295_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_295_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_295_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_295_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [296] !=: (Row -> Row1) # (Row -> Row1) -> Bool
  static inline void rewr_296_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150600c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [295] ==: (Row -> Row1) # (Row -> Row1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_295_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Row. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_166_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(16)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(17), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Row. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[295], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_295_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_295_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_295_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_295_2(result, t[0], t[1], this_rewriter); }


  // [295] ==: (Row -> Row1) # (Row -> Row1) -> Bool
  static inline void rewr_295_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069720));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [286] if: Bool # (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Row1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_286_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[286], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_286_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_286_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_286_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_286_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [286] if: Bool # (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Row1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_286_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[286], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_286_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_286_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_286_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_286_3(result, t[0], t[1], t[2], this_rewriter); }


  // [286] if: Bool # (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Row1 # Row -> Row
  static inline void rewr_286_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2e10));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [285] !=: (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_285_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_284_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[285], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_285_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_285_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_285_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_285_2(result, t[0], t[1], this_rewriter); }


  // [284] ==: (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_284_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_284_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_284_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_284_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [285] !=: (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Bool
  static inline void rewr_285_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085890));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [284] ==: (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_284_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Row1, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(27)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(19), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Row1, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[284], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_284_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_284_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_284_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_284_2(result, t[0], t[1], this_rewriter); }


  // [284] ==: (Row1 # Row -> Row) # (Row1 # Row -> Row) -> Bool
  static inline void rewr_284_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3280));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [279] if: Bool # (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Board1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_279_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[279], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_279_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_279_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_279_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_279_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [279] if: Bool # (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Board1 # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_279_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[279], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_279_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_279_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_279_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_279_3(result, t[0], t[1], t[2], this_rewriter); }


  // [279] if: Bool # (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Board1 # Row -> Row
  static inline void rewr_279_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa4a10));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [278] !=: (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_278_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_277_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[278], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_278_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_278_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_278_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_278_2(result, t[0], t[1], this_rewriter); }


  // [277] ==: (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_277_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_277_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_277_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_277_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [278] !=: (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Bool
  static inline void rewr_278_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068f00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [277] ==: (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_277_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Board1, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(20), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Board1, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[277], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_277_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_277_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_277_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_277_2(result, t[0], t[1], this_rewriter); }


  // [277] ==: (Board1 # Row -> Row) # (Board1 # Row -> Row) -> Bool
  static inline void rewr_277_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084d60));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [272] if: Bool # (Board -> Row) # (Board -> Row) -> Board -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_272_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[272], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_272_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_272_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_272_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_272_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [272] if: Bool # (Board -> Row) # (Board -> Row) -> Board -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_272_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[272], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_272_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_272_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_272_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_272_3(result, t[0], t[1], t[2], this_rewriter); }


  // [272] if: Bool # (Board -> Row) # (Board -> Row) -> Board -> Row
  static inline void rewr_272_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3180));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [271] !=: (Board -> Row) # (Board -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_271_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_270_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[271], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_271_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_271_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_271_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_271_2(result, t[0], t[1], this_rewriter); }


  // [270] ==: (Board -> Row) # (Board -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_270_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_270_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_270_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_270_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [271] !=: (Board -> Row) # (Board -> Row) -> Bool
  static inline void rewr_271_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085120));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [270] ==: (Board -> Row) # (Board -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_270_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Board. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(30)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(30)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(21), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Board. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[270], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_270_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_270_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_270_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_270_2(result, t[0], t[1], this_rewriter); }


  // [270] ==: (Board -> Row) # (Board -> Row) -> Bool
  static inline void rewr_270_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068810));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [261] if: Bool # (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_261_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[261], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_261_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_261_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_261_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_261_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [261] if: Bool # (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_261_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[261], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_261_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_261_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_261_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_261_3(result, t[0], t[1], t[2], this_rewriter); }


  // [261] if: Bool # (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Board1 # Board1 -> Board1
  static inline void rewr_261_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c31e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [260] !=: (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_260_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_259_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[260], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_260_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_260_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_260_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_260_2(result, t[0], t[1], this_rewriter); }


  // [259] ==: (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_259_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_259_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_259_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_259_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [260] !=: (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Bool
  static inline void rewr_260_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa5ba0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [259] ==: (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_259_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Board1. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_160_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(31)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(31)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(22), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Board1. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[259], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_259_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_259_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_259_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_259_2(result, t[0], t[1], this_rewriter); }


  // [259] ==: (Board1 # Board1 -> Board1) # (Board1 # Board1 -> Board1) -> Bool
  static inline void rewr_259_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069610));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [254] if: Bool # (Board -> Board1) # (Board -> Board1) -> Board -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3>
  static inline void rewr_254_4(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@R(@var_0(@var_2)))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@R(@var_1(@var_2)))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        make_application(result,var_1, local_rewrite(var_2));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[254], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_254_4_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_254_4(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_254_4_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_254_4(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], this_rewriter); }


  // [254] if: Bool # (Board -> Board1) # (Board -> Board1) -> Board -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_254_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[254], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_254_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_254_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_254_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_254_3(result, t[0], t[1], t[2], this_rewriter); }


  // [254] if: Bool # (Board -> Board1) # (Board -> Board1) -> Board -> Board1
  static inline void rewr_254_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3150));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [253] !=: (Board -> Board1) # (Board -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_253_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_252_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[253], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_253_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_253_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_253_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_253_2(result, t[0], t[1], this_rewriter); }


  // [252] ==: (Board -> Board1) # (Board -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_252_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_252_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_252_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_252_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [253] !=: (Board -> Board1) # (Board -> Board1) -> Bool
  static inline void rewr_253_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115082860));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [252] ==: (Board -> Board1) # (Board -> Board1) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_252_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Board. @var_0(x1) == @var_1(x1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_160_2(result, delayed_application1<data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(30)), this_rewriter), delayed_application1<data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(30)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(21), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Board. @var_0(x1) == @var_1(x1)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[252], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_252_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_252_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_252_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_252_2(result, t[0], t[1], this_rewriter); }


  // [252] ==: (Board -> Board1) # (Board -> Board1) -> Bool
  static inline void rewr_252_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c26b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [243] if: Bool # (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Board1 # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_243_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[243], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_243_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_243_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_243_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_243_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [243] if: Bool # (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Board1 # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_243_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[243], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_243_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_243_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_243_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_243_3(result, t[0], t[1], t[2], this_rewriter); }


  // [243] if: Bool # (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Board1 # Board -> Board
  static inline void rewr_243_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115085610));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [242] !=: (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_242_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_241_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[242], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_242_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_242_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_242_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_242_2(result, t[0], t[1], this_rewriter); }


  // [241] ==: (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_241_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_241_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_241_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_241_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [242] !=: (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Bool
  static inline void rewr_242_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068840));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [241] ==: (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_241_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Board1, x3: Board. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_182_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(32)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(29)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(32)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(23), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Board1, x3: Board. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[241], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_241_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_241_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_241_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_241_2(result, t[0], t[1], this_rewriter); }


  // [241] ==: (Board1 # Board -> Board) # (Board1 # Board -> Board) -> Bool
  static inline void rewr_241_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3480));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [236] if: Bool # (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Pos # Pos # Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5, class DATA_EXPR6>
  static inline void rewr_236_7(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, const DATA_EXPR6& arg_not_nf6, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_1(@var_2, @var_3, @var_4, @var_5)))))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_0(@var_2, @var_3, @var_4, @var_5)))))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4, @var_5)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_1(@var_2, @var_3, @var_4, @var_5)))))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[236], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf6); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_236_7_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_236_7(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), this_rewriter); }

  static inline void rewr_236_7_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_236_7(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], t[3], this_rewriter); }


  // [236] if: Bool # (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Pos # Pos # Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_236_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[236], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_236_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_236_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_236_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_236_3(result, t[0], t[1], t[2], this_rewriter); }


  // [236] if: Bool # (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Pos # Pos # Pos # Pos -> Bool
  static inline void rewr_236_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150688e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [235] !=: (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_235_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_234_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[235], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_235_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_235_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_235_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_235_2(result, t[0], t[1], this_rewriter); }


  // [234] ==: (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_234_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_234_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_234_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_234_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [235] !=: (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Bool
  static inline void rewr_235_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c31b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [234] ==: (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_234_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2,x3,x7: Pos. @var_0(x1, x2, x3, x7) == @var_1(x1, x2, x3, x7)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application4<data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(34)), this_rewriter), delayed_application4<data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(34)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(24), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2,x3,x7: Pos. @var_0(x1, x2, x3, x7) == @var_1(x1, x2, x3, x7)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[234], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_234_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_234_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_234_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_234_2(result, t[0], t[1], this_rewriter); }


  // [234] ==: (Pos # Pos # Pos # Pos -> Bool) # (Pos # Pos # Pos # Pos -> Bool) -> Bool
  static inline void rewr_234_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c33d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [229] if: Bool # (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Pos # Pos # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_229_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[229], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_229_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_229_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_229_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_229_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [229] if: Bool # (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Pos # Pos # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_229_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[229], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_229_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_229_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_229_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_229_3(result, t[0], t[1], t[2], this_rewriter); }


  // [229] if: Bool # (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Pos # Pos # Board -> Bool
  static inline void rewr_229_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068fd0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [228] !=: (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_228_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_227_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[228], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_228_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_228_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_228_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_228_2(result, t[0], t[1], this_rewriter); }


  // [227] ==: (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_227_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_227_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_227_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_227_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [228] !=: (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Bool
  static inline void rewr_228_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3250));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [227] ==: (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_227_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2: Pos, x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(25), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2: Pos, x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[227], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_227_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_227_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_227_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_227_2(result, t[0], t[1], this_rewriter); }


  // [227] ==: (Pos # Pos # Board -> Bool) # (Pos # Pos # Board -> Bool) -> Bool
  static inline void rewr_227_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068fa0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [222] if: Bool # (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Pos # Piece # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_222_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[222], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_222_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_222_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_222_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_222_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [222] if: Bool # (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Pos # Piece # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_222_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[222], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_222_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_222_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_222_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_222_3(result, t[0], t[1], t[2], this_rewriter); }


  // [222] if: Bool # (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Pos # Piece # Row -> Row
  static inline void rewr_222_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e11505d310));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [221] !=: (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_221_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_220_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[221], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_221_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_221_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_221_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_221_2(result, t[0], t[1], this_rewriter); }


  // [220] ==: (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_220_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_220_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_220_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_220_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [221] !=: (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Bool
  static inline void rewr_221_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084980));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [220] ==: (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_220_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Pos, x2: Piece, x5: Row. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_176_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(6)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(1)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(6)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(26), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Pos, x2: Piece, x5: Row. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[220], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_220_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_220_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_220_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_220_2(result, t[0], t[1], this_rewriter); }


  // [220] ==: (Pos # Piece # Row -> Row) # (Pos # Piece # Row -> Row) -> Bool
  static inline void rewr_220_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067980));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [215] if: Bool # (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Pos # Row -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_215_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[215], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_215_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_215_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_215_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_215_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [215] if: Bool # (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Pos # Row -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_215_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[215], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_215_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_215_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_215_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_215_3(result, t[0], t[1], t[2], this_rewriter); }


  // [215] if: Bool # (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Pos # Row -> Piece
  static inline void rewr_215_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068ed0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [214] !=: (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_214_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_213_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[214], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_214_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_214_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_214_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_214_2(result, t[0], t[1], this_rewriter); }


  // [213] ==: (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_213_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_213_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_213_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_213_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [214] !=: (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Bool
  static inline void rewr_214_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150681b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [213] ==: (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_213_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1: Pos, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_59_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(17)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(27), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1: Pos, x3: Row. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[213], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_213_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_213_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_213_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_213_2(result, t[0], t[1], this_rewriter); }


  // [213] ==: (Pos # Row -> Piece) # (Pos # Row -> Piece) -> Bool
  static inline void rewr_213_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150681e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [208] if: Bool # (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Pos # Pos # Piece # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5, class DATA_EXPR6>
  static inline void rewr_208_7(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, const DATA_EXPR6& arg_not_nf6, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_1(@var_2, @var_3, @var_4, @var_5)))))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_0(@var_2, @var_3, @var_4, @var_5)))))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4, @var_5)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@N(@@S(@var_5, @@R(@var_1(@var_2, @var_3, @var_4, @var_5)))))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        const DATA_EXPR6& var_5 = arg_not_nf6; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4), local_rewrite(var_5));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4, @var_5)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[208], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf6); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_208_7_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_208_7(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), this_rewriter); }

  static inline void rewr_208_7_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_208_7(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], t[3], this_rewriter); }


  // [208] if: Bool # (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Pos # Pos # Piece # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_208_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[208], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_208_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_208_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_208_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_208_3(result, t[0], t[1], t[2], this_rewriter); }


  // [208] if: Bool # (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Pos # Pos # Piece # Board -> Board
  static inline void rewr_208_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068910));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [207] !=: (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_207_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_206_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[207], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_207_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_207_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_207_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_207_2(result, t[0], t[1], this_rewriter); }


  // [206] ==: (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_206_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_206_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_206_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_206_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [207] !=: (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Bool
  static inline void rewr_207_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150688b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [206] ==: (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_206_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2: Pos, x3: Piece, x7: Board. @var_0(x1, x2, x3, x7) == @var_1(x1, x2, x3, x7)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_182_2(result, delayed_application4<data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(35)), this_rewriter), delayed_application4<data_expression,data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(13)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(35)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(28), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2: Pos, x3: Piece, x7: Board. @var_0(x1, x2, x3, x7) == @var_1(x1, x2, x3, x7)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[206], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_206_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_206_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_206_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_206_2(result, t[0], t[1], this_rewriter); }


  // [206] ==: (Pos # Pos # Piece # Board -> Board) # (Pos # Pos # Piece # Board -> Board) -> Bool
  static inline void rewr_206_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e114fa0fd0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [201] if: Bool # (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Pos # Pos # Board -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4, class DATA_EXPR5>
  static inline void rewr_201_6(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, const DATA_EXPR5& arg_not_nf5, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_0(@var_2, @var_3, @var_4)))))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3, @var_4)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@N(@@S(@var_4, @@R(@var_1(@var_2, @var_3, @var_4)))))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        const DATA_EXPR5& var_4 = arg_not_nf5; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3), local_rewrite(var_4));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3, @var_4)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[201], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf5); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_201_6_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_201_6(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_201_6_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_201_6(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], t[2], this_rewriter); }


  // [201] if: Bool # (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Pos # Pos # Board -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_201_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[201], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_201_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_201_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_201_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_201_3(result, t[0], t[1], t[2], this_rewriter); }


  // [201] if: Bool # (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Pos # Pos # Board -> Piece
  static inline void rewr_201_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068180));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [200] !=: (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_200_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_199_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[200], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_200_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_200_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_200_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_200_2(result, t[0], t[1], this_rewriter); }


  // [199] ==: (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_199_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_199_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_199_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_199_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [200] !=: (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Bool
  static inline void rewr_200_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067a80));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [199] ==: (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_199_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x2: Pos, x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_59_2(result, delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), delayed_application3<data_expression,data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(33)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(8)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(25), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x2: Pos, x5: Board. @var_0(x1, x2, x5) == @var_1(x1, x2, x5)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[199], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_199_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_199_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_199_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_199_2(result, t[0], t[1], this_rewriter); }


  // [199] ==: (Pos # Pos # Board -> Piece) # (Pos # Pos # Board -> Piece) -> Bool
  static inline void rewr_199_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3120));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [194] if: Bool # (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_194_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_0(@var_2, @var_3)))))))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,local_rewrite(var_1), local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
          const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
          make_application(result,local_rewrite(var_0), local_rewrite(var_2), local_rewrite(var_3));
          rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0(@var_2, @var_3)
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@N(@@S(@var_2, @@N(@@S(@var_3, @@R(@var_1(@var_2, @var_3)))))), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        const DATA_EXPR3& var_2 = arg_not_nf3; // S1b
        const DATA_EXPR4& var_3 = arg_not_nf4; // S1b
        make_application(result,var_1, local_rewrite(var_2), local_rewrite(var_3));
        rewrite_with_arguments_in_normal_form(result, result, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1(@var_2, @var_3)
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[194], arg0, arg1, arg2);
    make_application(result, result, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_194_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_194_5(result, term_not_in_normal_form(down_cast<application>(t.head())[0], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[1], this_rewriter), term_not_in_normal_form(down_cast<application>(t.head())[2], this_rewriter), term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_194_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_194_5(result, down_cast<application>(t.head())[0], down_cast<application>(t.head())[1], down_cast<application>(t.head())[2], t[0], t[1], this_rewriter); }


  // [194] if: Bool # (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_194_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[194], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_194_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_194_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_194_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_194_3(result, t[0], t[1], t[2], this_rewriter); }


  // [194] if: Bool # (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Pos # Pos -> Bool
  static inline void rewr_194_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067950));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [193] !=: (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_193_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_192_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[193], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_193_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_193_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_193_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_193_2(result, t[0], t[1], this_rewriter); }


  // [192] ==: (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_192_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_192_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_192_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_192_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [193] !=: (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Bool
  static inline void rewr_193_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2640));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [192] ==: (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_192_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(forall x1,x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_66_2(result, delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_0), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), delayed_application2<data_expression,data_expression,data_expression>(local_rewrite(var_1), static_cast<const data_expression&>(this_rewriter->bound_variable_get(25)), static_cast<const data_expression&>(this_rewriter->bound_variable_get(20)), this_rewriter), this_rewriter);
      this_rewriter->universal_quantifier_enumeration(result, this_rewriter->binding_variable_list_get(13), result, true, sigma(this_rewriter));
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 forall x1,x3: Pos. @var_0(x1, x3) == @var_1(x1, x3)
    }
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[192], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_192_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_192_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_192_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_192_2(result, t[0], t[1], this_rewriter); }


  // [192] ==: (Pos # Pos -> Bool) # (Pos # Pos -> Bool) -> Bool
  static inline void rewr_192_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c34b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [189] if: Bool # @NatPair # @NatPair -> @NatPair
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_189_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[189], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_189_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_189_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_189_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_189_3(result, t[0], t[1], t[2], this_rewriter); }


  // [189] if: Bool # @NatPair # @NatPair -> @NatPair
  static inline void rewr_189_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069000));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [188] !=: @NatPair # @NatPair -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_188_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_150_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[188], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_188_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_188_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_188_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_188_2(result, t[0], t[1], this_rewriter); }


  // [150] ==: @NatPair # @NatPair -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_150_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_150_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_150_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_150_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [188] !=: @NatPair # @NatPair -> Bool
  static inline void rewr_188_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3040));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [150] ==: @NatPair # @NatPair -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_150_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(@cPair, @@S(@var_1, @@N(@@S(@var_2, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cPair, @@S(@var_3, @@N(@@S(@var_4, @@R(@var_1 == @var_3 && @var_2 == @var_4)))), @@X))))))), @@N(@@M(@var_0, @@R(true), @@X))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f61be0) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f61be0) // F1
          {
            const data_expression& var_3 = down_cast<data_expression>(arg1[1]); // S2
            const data_expression& var_4 = down_cast<data_expression>(arg1[2]); // S2
            rewr_46_2(result, delayed_rewr_97_2<data_expression, data_expression>(var_1, var_3, this_rewriter), delayed_rewr_97_2<data_expression, data_expression>(var_2, var_4, this_rewriter), this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_1 == @var_3 && @var_2 == @var_4
          }
          else
          {
          }
        }
      }
      else
      {
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[150], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_150_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_150_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_150_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_150_2(result, t[0], t[1], this_rewriter); }


  // [97] ==: Nat # Nat -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_97_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_97_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_97_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_97_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [150] ==: @NatPair # @NatPair -> Bool
  static inline void rewr_150_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084f90));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [183] !=: Board # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_183_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_182_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[183], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_183_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_183_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_183_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_183_2(result, t[0], t[1], this_rewriter); }


  // [182] ==: Board # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_182_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_182_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_182_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_182_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [183] !=: Board # Board -> Bool
  static inline void rewr_183_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3070));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [182] ==: Board # Board -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_182_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@S(@var_1, @@C(@to_pos(@var_0) != @to_pos(@var_1), @@R(false), @@C(@to_pos(@var_0) == @to_pos(@var_1), @@R(@equal_arguments(@var_0, @var_1)), @@M(@var_0, @@R(true), @@X))))))
    {
      const data_expression& var_0 = arg0; // S1a
      const data_expression& var_1 = arg1; // S1a
      rewr_147_2(result, delayed_rewr_492_1<data_expression>(var_0, this_rewriter), delayed_rewr_492_1<data_expression>(var_1, this_rewriter), this_rewriter);
      if (result == sort_bool::true_()) // C
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 false
      }
      else
      {
        rewr_60_2(result, delayed_rewr_492_1<data_expression>(var_0, this_rewriter), delayed_rewr_492_1<data_expression>(var_1, this_rewriter), this_rewriter);
        if (result == sort_bool::true_()) // C
        {
          rewr_493_2(result, var_0, var_1, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @equal_arguments(@var_0, @var_1)
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[182], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_182_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_182_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_182_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_182_2(result, t[0], t[1], this_rewriter); }


  // [492] @to_pos: Board -> Pos
  template < class DATA_EXPR0>
  class delayed_rewr_492_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_492_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_492_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_492_1(result, m_t0, this_rewriter);
      }
  };
  
  // [182] ==: Board # Board -> Bool
  static inline void rewr_182_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066af0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [49] if: Bool # Board # Board -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_49_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[49], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_49_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_49_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_49_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_49_3(result, t[0], t[1], t[2], this_rewriter); }


  // [49] if: Bool # Board # Board -> Board
  static inline void rewr_49_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3940));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [177] !=: Row # Row -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_177_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_176_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[177], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_177_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_177_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_177_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_177_2(result, t[0], t[1], this_rewriter); }


  // [177] !=: Row # Row -> Bool
  static inline void rewr_177_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3760));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [176] ==: Row # Row -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_176_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@S(@var_1, @@C(@to_pos(@var_0) != @to_pos(@var_1), @@R(false), @@C(@to_pos(@var_0) == @to_pos(@var_1), @@R(@equal_arguments(@var_0, @var_1)), @@M(@var_0, @@R(true), @@X))))))
    {
      const data_expression& var_0 = arg0; // S1a
      const data_expression& var_1 = arg1; // S1a
      rewr_147_2(result, delayed_rewr_477_1<data_expression>(var_0, this_rewriter), delayed_rewr_477_1<data_expression>(var_1, this_rewriter), this_rewriter);
      if (result == sort_bool::true_()) // C
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 false
      }
      else
      {
        rewr_60_2(result, delayed_rewr_477_1<data_expression>(var_0, this_rewriter), delayed_rewr_477_1<data_expression>(var_1, this_rewriter), this_rewriter);
        if (result == sort_bool::true_()) // C
        {
          rewr_478_2(result, var_0, var_1, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @equal_arguments(@var_0, @var_1)
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[176], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_176_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_176_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_176_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_176_2(result, t[0], t[1], this_rewriter); }


  // [477] @to_pos: Row -> Pos
  template < class DATA_EXPR0>
  class delayed_rewr_477_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_477_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_477_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_477_1(result, m_t0, this_rewriter);
      }
  };
  
  // [176] ==: Row # Row -> Bool
  static inline void rewr_176_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3010));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [50] if: Bool # Row # Row -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_50_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[50], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_50_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_50_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_50_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_50_3(result, t[0], t[1], t[2], this_rewriter); }


  // [50] if: Bool # Row # Row -> Row
  static inline void rewr_50_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c39d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [59] ==: Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_59_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@S(@var_1, @@C(@to_pos(@var_0) != @to_pos(@var_1), @@R(false), @@C(@to_pos(@var_0) == @to_pos(@var_1), @@R(@equal_arguments(@var_0, @var_1)), @@M(@var_0, @@R(true), @@X))))))
    {
      const data_expression& var_0 = arg0; // S1a
      const data_expression& var_1 = arg1; // S1a
      rewr_147_2(result, delayed_rewr_462_1<data_expression>(var_0, this_rewriter), delayed_rewr_462_1<data_expression>(var_1, this_rewriter), this_rewriter);
      if (result == sort_bool::true_()) // C
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 false
      }
      else
      {
        rewr_60_2(result, delayed_rewr_462_1<data_expression>(var_0, this_rewriter), delayed_rewr_462_1<data_expression>(var_1, this_rewriter), this_rewriter);
        if (result == sort_bool::true_()) // C
        {
          rewr_463_2(result, var_0, var_1, this_rewriter);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @equal_arguments(@var_0, @var_1)
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[59], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_59_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_59_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_59_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_59_2(result, t[0], t[1], this_rewriter); }


  // [462] @to_pos: Piece -> Pos
  template < class DATA_EXPR0>
  class delayed_rewr_462_1
  {
    protected:
      const DATA_EXPR0& m_t0;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_462_1(const DATA_EXPR0& t0, RewriterCompilingJitty* tr)
        : m_t0(t0), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_462_1(local_store, m_t0, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_462_1(result, m_t0, this_rewriter);
      }
  };
  
  // [59] ==: Piece # Piece -> Bool
  static inline void rewr_59_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c39a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [52] if: Bool # Piece # Piece -> Piece
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_52_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[52], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_52_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_52_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_52_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_52_3(result, t[0], t[1], t[2], this_rewriter); }


  // [52] if: Bool # Piece # Piece -> Piece
  static inline void rewr_52_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3790));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [45] !=: Piece # Piece -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_45_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_59_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[45], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_45_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_45_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_45_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_45_2(result, t[0], t[1], this_rewriter); }


  // [45] !=: Piece # Piece -> Bool
  static inline void rewr_45_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c33a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [167] !=: Row1 # Row1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_167_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_166_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[167], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_167_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_167_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_167_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_167_2(result, t[0], t[1], this_rewriter); }


  // [166] ==: Row1 # Row1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_166_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_166_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_166_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_166_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [167] !=: Row1 # Row1 -> Bool
  static inline void rewr_167_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3820));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [166] ==: Row1 # Row1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_166_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[166], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_166_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_166_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_166_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_166_2(result, t[0], t[1], this_rewriter); }


  // [166] ==: Row1 # Row1 -> Bool
  static inline void rewr_166_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084f30));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [51] if: Bool # Row1 # Row1 -> Row1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_51_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[51], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_51_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_51_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_51_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_51_3(result, t[0], t[1], t[2], this_rewriter); }


  // [51] if: Bool # Row1 # Row1 -> Row1
  static inline void rewr_51_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066b50));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [161] !=: Board1 # Board1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_161_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_160_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[161], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_161_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_161_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_161_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_161_2(result, t[0], t[1], this_rewriter); }


  // [160] ==: Board1 # Board1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_160_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_160_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_160_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_160_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [161] !=: Board1 # Board1 -> Bool
  static inline void rewr_161_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115068f70));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [160] ==: Board1 # Board1 -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_160_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@X)))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[160], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_160_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_160_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_160_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_160_2(result, t[0], t[1], this_rewriter); }


  // [160] ==: Board1 # Board1 -> Bool
  static inline void rewr_160_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067370));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [48] if: Bool # Board1 # Board1 -> Board1
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_48_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[48], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_48_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_48_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_48_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_48_3(result, t[0], t[1], t[2], this_rewriter); }


  // [48] if: Bool # Board1 # Board1 -> Board1
  static inline void rewr_48_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c37f0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [147] !=: Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_147_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_60_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[147], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_147_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_147_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_147_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_147_2(result, t[0], t[1], this_rewriter); }


  // [60] ==: Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_60_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_60_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_60_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_60_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [147] !=: Pos # Pos -> Bool
  static inline void rewr_147_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3520));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [146] if: Bool # Pos # Pos -> Pos
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_146_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[146], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_146_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_146_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_146_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_146_3(result, t[0], t[1], t[2], this_rewriter); }


  // [146] if: Bool # Pos # Pos -> Pos
  static inline void rewr_146_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067310));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [60] ==: Pos # Pos -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_60_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(@cDub, @@S(@var_1, @@F(true, @@D(@@N(@@S(@var_2, @@D(@@N(@@M(@var_0, @@R(true), @@F(1, @@R(false), @@F(@cDub, @@M(@var_1, @@F(false, @@D(@@N(@@R(false))), @@N(@@S(@var_3, @@R(@var_2 == @var_3)))), @@F(false, @@D(@@N(@@R(false))), @@D(@@X))), @@X)))))))), @@F(false, @@D(@@N(@@S(@var_2, @@D(@@N(@@M(@var_0, @@R(true), @@F(1, @@R(false), @@F(@cDub, @@M(@var_1, @@F(true, @@D(@@N(@@R(false))), @@N(@@S(@var_3, @@R(@var_2 == @var_3)))), @@F(true, @@D(@@N(@@R(false))), @@D(@@X))), @@X)))))))), @@N(@@S(@var_2, @@D(@@N(@@M(@var_0, @@R(true), @@F(1, @@R(false), @@F(@cDub, @@M(@var_1, @@N(@@S(@var_3, @@R(@var_2 == @var_3))), @@D(@@X)), @@X)))))))))), @@F(1, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cDub, @@N(@@R(false)), @@X)))), @@N(@@M(@var_0, @@R(true), @@X)))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5e1c0) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        if (uint_address(arg0[1]) == 0x55e114f5e170) // F2a true
        {
          const data_expression& t1 = down_cast<data_expression>(arg0[1]);
          const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
            if (uint_address(arg1) == 0x55e114f5df68) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 false
            }
            else
            {
              if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e1c0) // F1
              {
                if (var_1 == arg1[1]) // M
                {
                  if (uint_address(arg1[1]) == 0x55e114f5e198) // F2a false
                  {
                    const data_expression& t3 = down_cast<data_expression>(arg1[1]);
                    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 false
                  }
                  else
                  {
                    const data_expression& var_3 = down_cast<data_expression>(arg1[2]); // S2
                    rewr_60_2(result, var_2, var_3, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 @var_2 == @var_3
                  }
                }
                else
                {
                  if (uint_address(arg1[1]) == 0x55e114f5e198) // F2a false
                  {
                    const data_expression& t3 = down_cast<data_expression>(arg1[1]);
                    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 false
                  }
                  else
                  {
                  }
                }
              }
              else
              {
              }
            }
          }
        }
        else
        {
          if (uint_address(arg0[1]) == 0x55e114f5e198) // F2a false
          {
            const data_expression& t1 = down_cast<data_expression>(arg0[1]);
            const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
            if (var_0 == arg1) // M
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 true
            }
            else
            {
              if (uint_address(arg1) == 0x55e114f5df68) // F1
              {
                result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                return; // R1 false
              }
              else
              {
                if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e1c0) // F1
                {
                  if (var_1 == arg1[1]) // M
                  {
                    if (uint_address(arg1[1]) == 0x55e114f5e170) // F2a true
                    {
                      const data_expression& t3 = down_cast<data_expression>(arg1[1]);
                      result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                      return; // R1 false
                    }
                    else
                    {
                      const data_expression& var_3 = down_cast<data_expression>(arg1[2]); // S2
                      rewr_60_2(result, var_2, var_3, this_rewriter);
                      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                      return; // R1 @var_2 == @var_3
                    }
                  }
                  else
                  {
                    if (uint_address(arg1[1]) == 0x55e114f5e170) // F2a true
                    {
                      const data_expression& t3 = down_cast<data_expression>(arg1[1]);
                      result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                      return; // R1 false
                    }
                    else
                    {
                    }
                  }
                }
                else
                {
                }
              }
            }
          }
          else
          {
            const data_expression& var_2 = down_cast<data_expression>(arg0[2]); // S2
            if (var_0 == arg1) // M
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 true
            }
            else
            {
              if (uint_address(arg1) == 0x55e114f5df68) // F1
              {
                result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                return; // R1 false
              }
              else
              {
                if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e1c0) // F1
                {
                  if (var_1 == arg1[1]) // M
                  {
                    const data_expression& var_3 = down_cast<data_expression>(arg1[2]); // S2
                    rewr_60_2(result, var_2, var_3, this_rewriter);
                    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                    return; // R1 @var_2 == @var_3
                  }
                  else
                  {
                  }
                }
                else
                {
                }
              }
            }
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5df68) // F1
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
            if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5e1c0) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 false
            }
            else
            {
            }
          }
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[60], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_60_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_60_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_60_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_60_2(result, t[0], t[1], this_rewriter); }


  // [60] ==: Pos # Pos -> Bool
  static inline void rewr_60_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066b20));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [153] !=: Nat # Nat -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_153_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_97_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[153], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_153_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_153_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_153_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_153_2(result, t[0], t[1], this_rewriter); }


  // [153] !=: Nat # Nat -> Bool
  static inline void rewr_153_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3880));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [107] if: Bool # Nat # Nat -> Nat
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_107_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[107], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_107_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_107_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_107_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_107_3(result, t[0], t[1], t[2], this_rewriter); }


  // [107] if: Bool # Nat # Nat -> Nat
  static inline void rewr_107_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c38e0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [97] ==: Nat # Nat -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_97_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(@cNat, @@S(@var_1, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cNat, @@S(@var_2, @@R(@var_1 == @var_2)), @@F(0, @@R(false), @@X)))))), @@F(0, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cNat, @@R(false), @@X)))), @@N(@@M(@var_0, @@R(true), @@X)))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5df90) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5df90) // F1
          {
            const data_expression& var_2 = down_cast<data_expression>(arg1[1]); // S2
            rewr_60_2(result, var_1, var_2, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_1 == @var_2
          }
          else
          {
            if (uint_address(arg1) == 0x55e114f61258) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 false
            }
            else
            {
            }
          }
        }
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f61258) // F1
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
            if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5df90) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 false
            }
            else
            {
            }
          }
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[97], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_97_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_97_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_97_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_97_2(result, t[0], t[1], this_rewriter); }


  // [97] ==: Nat # Nat -> Bool
  static inline void rewr_97_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c38b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [124] !=: Int # Int -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_124_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_34_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[124], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_124_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_124_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_124_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_124_2(result, t[0], t[1], this_rewriter); }


  // [34] ==: Int # Int -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_34_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_34_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_34_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_34_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [124] !=: Int # Int -> Bool
  static inline void rewr_124_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2fe0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [108] if: Bool # Int # Int -> Int
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_108_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[108], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_108_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_108_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_108_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_108_3(result, t[0], t[1], t[2], this_rewriter); }


  // [108] if: Bool # Int # Int -> Int
  static inline void rewr_108_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c2fb0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [34] ==: Int # Int -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_34_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@F(@cNeg, @@S(@var_1, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cNeg, @@S(@var_2, @@R(@var_1 == @var_2)), @@F(@cInt, @@R(false), @@X)))))), @@F(@cInt, @@S(@var_1, @@D(@@N(@@M(@var_0, @@R(true), @@F(@cInt, @@S(@var_2, @@R(@var_1 == @var_2)), @@F(@cNeg, @@R(false), @@X)))))), @@N(@@M(@var_0, @@R(true), @@X)))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f607b8) // F1
      {
        const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
        if (var_0 == arg1) // M
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
          if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f607b8) // F1
          {
            const data_expression& var_2 = down_cast<data_expression>(arg1[1]); // S2
            rewr_60_2(result, var_1, var_2, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 @var_1 == @var_2
          }
          else
          {
            if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5dfb8) // F1
            {
              result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 false
            }
            else
            {
            }
          }
        }
      }
      else
      {
        if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5dfb8) // F1
        {
          const data_expression& var_1 = down_cast<data_expression>(arg0[1]); // S2
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
            if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f5dfb8) // F1
            {
              const data_expression& var_2 = down_cast<data_expression>(arg1[1]); // S2
              rewr_97_2(result, var_1, var_2, this_rewriter);
              this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
              return; // R1 @var_1 == @var_2
            }
            else
            {
              if (uint_address((is_function_symbol(arg1) ? arg1 : arg1[0])) == 0x55e114f607b8) // F1
              {
                result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
                this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
                return; // R1 false
              }
              else
              {
              }
            }
          }
        }
        else
        {
          if (var_0 == arg1) // M
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[34], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_34_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_34_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_34_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_34_2(result, t[0], t[1], this_rewriter); }


  // [34] ==: Int # Int -> Bool
  static inline void rewr_34_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c37c0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [69] !=: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_69_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@S(@var_0, @@N(@@S(@var_1, @@R(!(@var_0 == @var_1)))))
    {
      const DATA_EXPR0& var_0 = arg_not_nf0; // S1b
      const DATA_EXPR1& var_1 = arg_not_nf1; // S1b
      rewr_61_1(result, delayed_rewr_66_2<DATA_EXPR0, DATA_EXPR1>(var_0, var_1, this_rewriter), this_rewriter);
      this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
      return; // R1 !(@var_0 == @var_1)
    }
    make_application(result, this_rewriter->normal_forms_for_constants[69], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_69_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_69_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_69_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_69_2(result, t[0], t[1], this_rewriter); }


  // [66] ==: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  class delayed_rewr_66_2
  {
    protected:
      const DATA_EXPR0& m_t0;
      const DATA_EXPR1& m_t1;
      RewriterCompilingJitty* this_rewriter;
    public:
      delayed_rewr_66_2(const DATA_EXPR0& t0, const DATA_EXPR1& t1, RewriterCompilingJitty* tr)
        : m_t0(t0), m_t1(t1), this_rewriter(tr)
      {}

      data_expression& normal_form() const
      {
        data_expression& local_store=this_rewriter->m_rewrite_stack.new_stack_position();
        rewr_66_2(local_store, m_t0, m_t1, this_rewriter);
        return local_store;
      }
      void normal_form(data_expression& result) const
      {
        rewr_66_2(result, m_t0, m_t1, this_rewriter);
      }
  };
  
  // [69] !=: Bool # Bool -> Bool
  static inline void rewr_69_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3a00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [66] ==: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_66_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@S(@var_0, @@R(!@var_0)))), @@F(true, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        rewr_61_1(result, var_0, this_rewriter);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 !@var_0
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@M(@var_0, @@R(true), @@F(true, @@R(@var_0), @@F(false, @@R(!@var_0), @@X)))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (var_0 == arg1) // M
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
        if (uint_address(arg1) == 0x55e114f5e170) // F1
        {
          result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
          if (uint_address(arg1) == 0x55e114f5e198) // F1
          {
            rewr_61_1(result, var_0, this_rewriter);
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 !@var_0
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[66], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_66_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_66_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_66_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_66_2(result, t[0], t[1], this_rewriter); }


  // [66] ==: Bool # Bool -> Bool
  static inline void rewr_66_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084f60));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [61] !: Bool -> Bool
  template < class DATA_EXPR0>
  static inline void rewr_61_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(!, @@S(@var_0, @@R(@var_0)), @@F(true, @@R(false), @@F(false, @@R(true), @@X)))
    {
      if (uint_address((is_function_symbol(arg0) ? arg0 : arg0[0])) == 0x55e114f5f548) // F1
      {
        const data_expression& var_0 = down_cast<data_expression>(arg0[1]); // S2
        result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 false
        }
        else
        {
          if (uint_address(arg0) == 0x55e114f5e198) // F1
          {
            result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
            this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
            return; // R1 true
          }
          else
          {
          }
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[61], arg0);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_61_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_61_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_61_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_61_1(result, t[0], this_rewriter); }


  // [61] !: Bool -> Bool
  static inline void rewr_61_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3910));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [47] ||: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_47_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@F(true, @@D(@@N(@@S(@var_0, @@R(true)))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
        local_rewrite(result, var_0);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 true
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@F(true, @@R(true), @@F(false, @@R(@var_0), @@X))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address(arg1) == 0x55e114f5e170) // F1
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 true
      }
      else
      {
        if (uint_address(arg1) == 0x55e114f5e198) // F1
        {
          result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[47], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_47_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_47_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_47_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_47_2(result, t[0], t[1], this_rewriter); }


  // [47] ||: Bool # Bool -> Bool
  static inline void rewr_47_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3970));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [46] &&: Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_46_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@S(@var_0, @@R(false)))), @@F(true, @@D(@@N(@@S(@var_0, @@R(@var_0)))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 false
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@S(@var_0, @@N(@@F(true, @@R(@var_0), @@F(false, @@R(false), @@X))))
    {
      const data_expression& var_0 = arg0; // S1a
      if (uint_address(arg1) == 0x55e114f5e170) // F1
      {
        result.assign(var_0, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_0
      }
      else
      {
        if (uint_address(arg1) == 0x55e114f5e198) // F1
        {
          result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 false
        }
        else
        {
        }
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[46], arg0, arg1);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_46_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_46_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_46_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_46_2(result, t[0], t[1], this_rewriter); }


  // [46] &&: Bool # Bool -> Bool
  static inline void rewr_46_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066b80));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [36] if: Bool # Bool # Bool -> Bool
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2>
  static inline void rewr_36_3(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    // @@A(0)
    data_expression& arg0(std::is_convertible<DATA_EXPR0, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf0))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR0, const data_expression&>::value)
    {
      local_rewrite(arg0, arg_not_nf0);
    }
    // Considering argument 0
    // @@F(false, @@D(@@N(@@N(@@S(@var_1, @@R(@var_1))))), @@F(true, @@D(@@N(@@S(@var_0, @@N(@@R(@var_0))))), @@X))
    {
      if (uint_address(arg0) == 0x55e114f5e198) // F1
      {
        const DATA_EXPR2& var_1 = arg_not_nf2; // S1b
        local_rewrite(result, var_1);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
        if (uint_address(arg0) == 0x55e114f5e170) // F1
        {
          const DATA_EXPR1& var_0 = arg_not_nf1; // S1b
          local_rewrite(result, var_0);
          this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
          return; // R1 @var_0
        }
        else
        {
        }
      }
    }
    // @@A(1)
    data_expression& arg1(std::is_convertible<DATA_EXPR1, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf1))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR1, const data_expression&>::value)
    {
      local_rewrite(arg1, arg_not_nf1);
    }
    // Considering argument 1
    // @@A(2)
    data_expression& arg2(std::is_convertible<DATA_EXPR2, const data_expression&>::value?(const_cast<data_expression&>(reinterpret_cast<const data_expression&>(arg_not_nf2))):this_rewriter->m_rewrite_stack.new_stack_position());
    if constexpr (!std::is_convertible<DATA_EXPR2, const data_expression&>::value)
    {
      local_rewrite(arg2, arg_not_nf2);
    }
    // Considering argument 2
    // @@N(@@S(@var_1, @@N(@@M(@var_1, @@R(@var_1), @@X))))
    {
      const data_expression& var_1 = arg1; // S1a
      if (var_1 == arg2) // M
      {
        result.assign(var_1, *this_rewriter->m_thread_aterm_pool);
        this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
        return; // R1 @var_1
      }
      else
      {
      }
    }
    make_application(result, this_rewriter->normal_forms_for_constants[36], arg0, arg1, arg2);
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_36_3_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_36_3(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), this_rewriter); }

  static inline void rewr_36_3_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_36_3(result, t[0], t[1], t[2], this_rewriter); }


  // [36] if: Bool # Bool # Bool -> Bool
  static inline void rewr_36_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115069030));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [1] c_row: Row1
  static inline void rewr_1_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063a00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [0] c_col: Board1
  static inline void rewr_0_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115063ff0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [40] col: Row # Row # Row # Row # Row -> Board
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_40_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[40], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf2); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_40_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_40_5(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), term_not_in_normal_form(t[4], this_rewriter), this_rewriter); }

  static inline void rewr_40_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_40_5(result, t[0], t[1], t[2], t[3], t[4], this_rewriter); }


  // [40] col: Row # Row # Row # Row # Row -> Board
  static inline void rewr_40_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150c3850));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [41] row: Piece # Piece # Piece # Piece # Piece -> Row
  template < class DATA_EXPR0, class DATA_EXPR1, class DATA_EXPR2, class DATA_EXPR3, class DATA_EXPR4>
  static inline void rewr_41_5(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, const DATA_EXPR2& arg_not_nf2, const DATA_EXPR3& arg_not_nf3, const DATA_EXPR4& arg_not_nf4, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[41], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf2); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf3); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf4); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_41_5_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_41_5(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), term_not_in_normal_form(t[2], this_rewriter), term_not_in_normal_form(t[3], this_rewriter), term_not_in_normal_form(t[4], this_rewriter), this_rewriter); }

  static inline void rewr_41_5_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_41_5(result, t[0], t[1], t[2], t[3], t[4], this_rewriter); }


  // [41] row: Piece # Piece # Piece # Piece # Piece -> Row
  static inline void rewr_41_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150662b0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [44] empty: Piece
  static inline void rewr_44_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150690a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [42] white: Piece
  static inline void rewr_42_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150673a0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [43] black: Piece
  static inline void rewr_43_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115084f00));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [39] @cDub: Bool # Pos -> Pos
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_39_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[39], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_39_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_39_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_39_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_39_2(result, t[0], t[1], this_rewriter); }


  // [39] @cDub: Bool # Pos -> Pos
  static inline void rewr_39_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066320));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [30] 1: Pos
  static inline void rewr_30_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081520));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [127] @cPair: Nat # Nat -> @NatPair
  template < class DATA_EXPR0, class DATA_EXPR1>
  static inline void rewr_127_2(data_expression& result, const DATA_EXPR0& arg_not_nf0, const DATA_EXPR1& arg_not_nf1, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[127], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); }, [&](data_expression& result){ local_rewrite(result, arg_not_nf1); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_127_2_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_127_2(result, term_not_in_normal_form(t[0], this_rewriter), term_not_in_normal_form(t[1], this_rewriter), this_rewriter); }

  static inline void rewr_127_2_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_127_2(result, t[0], t[1], this_rewriter); }


  // [127] @cPair: Nat # Nat -> @NatPair
  static inline void rewr_127_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e1150663d0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [109] 0: Nat
  static inline void rewr_109_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115066440));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [31] @cNat: Pos -> Nat
  template < class DATA_EXPR0>
  static inline void rewr_31_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[31], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_31_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_31_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_31_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_31_1(result, t[0], this_rewriter); }


  // [31] @cNat: Pos -> Nat
  static inline void rewr_31_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115065b70));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [72] @cNeg: Pos -> Int
  template < class DATA_EXPR0>
  static inline void rewr_72_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[72], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_72_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_72_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_72_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_72_1(result, t[0], this_rewriter); }


  // [72] @cNeg: Pos -> Int
  static inline void rewr_72_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115065be0));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [32] @cInt: Nat -> Int
  template < class DATA_EXPR0>
  static inline void rewr_32_1(data_expression& result, const DATA_EXPR0& arg_not_nf0, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    make_application(result, this_rewriter->normal_forms_for_constants[32], [&](data_expression& result){ local_rewrite(result, arg_not_nf0); });
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }

  static inline void rewr_32_1_term(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_32_1(result, term_not_in_normal_form(t[0], this_rewriter), this_rewriter); }

  static inline void rewr_32_1_term_arg_in_normal_form(data_expression& result, const application& t, RewriterCompilingJitty* this_rewriter) { rewr_32_1(result, t[0], this_rewriter); }


  // [32] @cInt: Nat -> Int
  static inline void rewr_32_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115065c50));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [38] false: Bool
  static inline void rewr_38_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115067340));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


  // [37] true: Bool
  static inline void rewr_37_0(data_expression& result, RewriterCompilingJitty* this_rewriter)
  {
    mcrl2::utilities::mcrl2_unused(this_rewriter); // Suppress warning
    std::size_t old_stack_size=this_rewriter->m_rewrite_stack.stack_size();
    result.unprotected_assign<false>(*reinterpret_cast<const data_expression*>(0x55e115081400));
    this_rewriter->m_rewrite_stack.reset_stack_size(old_stack_size);
    return;
  }


};
} // namespace
void set_the_precompiled_rewrite_functions_in_a_lookup_table(RewriterCompilingJitty* this_rewriter)
{
  assert(&this_rewriter->functions_when_arguments_are_not_in_normal_form == (void *)0x55e1150161b8);  // Check that this table matches the one rewriter is actually using.
  assert(&this_rewriter->functions_when_arguments_are_in_normal_form == (void *)0x55e1150161d0);  // Check that this table matches the one rewriter is actually using.
  for(rewriter_function& f: this_rewriter->functions_when_arguments_are_not_in_normal_form)
  {
    f = nullptr;
  }
  for(rewriter_function& f: this_rewriter->functions_when_arguments_are_in_normal_form)
  {
    f = nullptr;
  }
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 4 + 3] = rewr_functions::rewr_4_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 4 + 3] = rewr_functions::rewr_4_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 5 + 4] = rewr_functions::rewr_5_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 5 + 4] = rewr_functions::rewr_5_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 6 + 2] = rewr_functions::rewr_6_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 6 + 2] = rewr_functions::rewr_6_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 7 + 3] = rewr_functions::rewr_7_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 7 + 3] = rewr_functions::rewr_7_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 12 + 2] = rewr_functions::rewr_12_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 12 + 2] = rewr_functions::rewr_12_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 13 + 1] = rewr_functions::rewr_13_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 13 + 1] = rewr_functions::rewr_13_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 14 + 2] = rewr_functions::rewr_14_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 14 + 2] = rewr_functions::rewr_14_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 15 + 1] = rewr_functions::rewr_15_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 15 + 1] = rewr_functions::rewr_15_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 16 + 1] = rewr_functions::rewr_16_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 16 + 1] = rewr_functions::rewr_16_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 17 + 1] = rewr_functions::rewr_17_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 17 + 1] = rewr_functions::rewr_17_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 18 + 1] = rewr_functions::rewr_18_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 18 + 1] = rewr_functions::rewr_18_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 19 + 1] = rewr_functions::rewr_19_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 19 + 1] = rewr_functions::rewr_19_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 20 + 2] = rewr_functions::rewr_20_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 20 + 2] = rewr_functions::rewr_20_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 21 + 2] = rewr_functions::rewr_21_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 21 + 2] = rewr_functions::rewr_21_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 22 + 1] = rewr_functions::rewr_22_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 22 + 1] = rewr_functions::rewr_22_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 23 + 2] = rewr_functions::rewr_23_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 23 + 2] = rewr_functions::rewr_23_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 24 + 1] = rewr_functions::rewr_24_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 24 + 1] = rewr_functions::rewr_24_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 25 + 1] = rewr_functions::rewr_25_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 25 + 1] = rewr_functions::rewr_25_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 26 + 1] = rewr_functions::rewr_26_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 26 + 1] = rewr_functions::rewr_26_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 27 + 1] = rewr_functions::rewr_27_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 27 + 1] = rewr_functions::rewr_27_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 28 + 1] = rewr_functions::rewr_28_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 28 + 1] = rewr_functions::rewr_28_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 29 + 2] = rewr_functions::rewr_29_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 29 + 2] = rewr_functions::rewr_29_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 31 + 1] = rewr_functions::rewr_31_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 31 + 1] = rewr_functions::rewr_31_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 32 + 1] = rewr_functions::rewr_32_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 32 + 1] = rewr_functions::rewr_32_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 34 + 2] = rewr_functions::rewr_34_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 34 + 2] = rewr_functions::rewr_34_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 36 + 3] = rewr_functions::rewr_36_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 36 + 3] = rewr_functions::rewr_36_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 39 + 2] = rewr_functions::rewr_39_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 39 + 2] = rewr_functions::rewr_39_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 40 + 5] = rewr_functions::rewr_40_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 40 + 5] = rewr_functions::rewr_40_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 41 + 5] = rewr_functions::rewr_41_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 41 + 5] = rewr_functions::rewr_41_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 45 + 2] = rewr_functions::rewr_45_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 45 + 2] = rewr_functions::rewr_45_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 46 + 2] = rewr_functions::rewr_46_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 46 + 2] = rewr_functions::rewr_46_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 47 + 2] = rewr_functions::rewr_47_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 47 + 2] = rewr_functions::rewr_47_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 48 + 3] = rewr_functions::rewr_48_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 48 + 3] = rewr_functions::rewr_48_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 49 + 3] = rewr_functions::rewr_49_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 49 + 3] = rewr_functions::rewr_49_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 50 + 3] = rewr_functions::rewr_50_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 50 + 3] = rewr_functions::rewr_50_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 51 + 3] = rewr_functions::rewr_51_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 51 + 3] = rewr_functions::rewr_51_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 52 + 3] = rewr_functions::rewr_52_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 52 + 3] = rewr_functions::rewr_52_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 59 + 2] = rewr_functions::rewr_59_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 59 + 2] = rewr_functions::rewr_59_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 60 + 2] = rewr_functions::rewr_60_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 60 + 2] = rewr_functions::rewr_60_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 61 + 1] = rewr_functions::rewr_61_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 61 + 1] = rewr_functions::rewr_61_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 66 + 2] = rewr_functions::rewr_66_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 66 + 2] = rewr_functions::rewr_66_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 69 + 2] = rewr_functions::rewr_69_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 69 + 2] = rewr_functions::rewr_69_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 72 + 1] = rewr_functions::rewr_72_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 72 + 1] = rewr_functions::rewr_72_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 97 + 2] = rewr_functions::rewr_97_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 97 + 2] = rewr_functions::rewr_97_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 107 + 3] = rewr_functions::rewr_107_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 107 + 3] = rewr_functions::rewr_107_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 108 + 3] = rewr_functions::rewr_108_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 108 + 3] = rewr_functions::rewr_108_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 124 + 2] = rewr_functions::rewr_124_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 124 + 2] = rewr_functions::rewr_124_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 127 + 2] = rewr_functions::rewr_127_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 127 + 2] = rewr_functions::rewr_127_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 146 + 3] = rewr_functions::rewr_146_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 146 + 3] = rewr_functions::rewr_146_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 147 + 2] = rewr_functions::rewr_147_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 147 + 2] = rewr_functions::rewr_147_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 150 + 2] = rewr_functions::rewr_150_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 150 + 2] = rewr_functions::rewr_150_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 153 + 2] = rewr_functions::rewr_153_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 153 + 2] = rewr_functions::rewr_153_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 160 + 2] = rewr_functions::rewr_160_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 160 + 2] = rewr_functions::rewr_160_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 161 + 2] = rewr_functions::rewr_161_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 161 + 2] = rewr_functions::rewr_161_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 166 + 2] = rewr_functions::rewr_166_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 166 + 2] = rewr_functions::rewr_166_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 167 + 2] = rewr_functions::rewr_167_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 167 + 2] = rewr_functions::rewr_167_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 176 + 2] = rewr_functions::rewr_176_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 176 + 2] = rewr_functions::rewr_176_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 177 + 2] = rewr_functions::rewr_177_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 177 + 2] = rewr_functions::rewr_177_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 182 + 2] = rewr_functions::rewr_182_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 182 + 2] = rewr_functions::rewr_182_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 183 + 2] = rewr_functions::rewr_183_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 183 + 2] = rewr_functions::rewr_183_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 188 + 2] = rewr_functions::rewr_188_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 188 + 2] = rewr_functions::rewr_188_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 189 + 3] = rewr_functions::rewr_189_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 189 + 3] = rewr_functions::rewr_189_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 192 + 2] = rewr_functions::rewr_192_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 192 + 2] = rewr_functions::rewr_192_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 193 + 2] = rewr_functions::rewr_193_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 193 + 2] = rewr_functions::rewr_193_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 194 + 3] = rewr_functions::rewr_194_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 194 + 3] = rewr_functions::rewr_194_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 194 + 5] = rewr_functions::rewr_194_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 194 + 5] = rewr_functions::rewr_194_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 199 + 2] = rewr_functions::rewr_199_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 199 + 2] = rewr_functions::rewr_199_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 200 + 2] = rewr_functions::rewr_200_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 200 + 2] = rewr_functions::rewr_200_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 201 + 3] = rewr_functions::rewr_201_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 201 + 3] = rewr_functions::rewr_201_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 201 + 6] = rewr_functions::rewr_201_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 201 + 6] = rewr_functions::rewr_201_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 206 + 2] = rewr_functions::rewr_206_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 206 + 2] = rewr_functions::rewr_206_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 207 + 2] = rewr_functions::rewr_207_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 207 + 2] = rewr_functions::rewr_207_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 208 + 3] = rewr_functions::rewr_208_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 208 + 3] = rewr_functions::rewr_208_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 208 + 7] = rewr_functions::rewr_208_7_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 208 + 7] = rewr_functions::rewr_208_7_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 213 + 2] = rewr_functions::rewr_213_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 213 + 2] = rewr_functions::rewr_213_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 214 + 2] = rewr_functions::rewr_214_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 214 + 2] = rewr_functions::rewr_214_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 215 + 3] = rewr_functions::rewr_215_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 215 + 3] = rewr_functions::rewr_215_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 215 + 5] = rewr_functions::rewr_215_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 215 + 5] = rewr_functions::rewr_215_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 220 + 2] = rewr_functions::rewr_220_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 220 + 2] = rewr_functions::rewr_220_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 221 + 2] = rewr_functions::rewr_221_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 221 + 2] = rewr_functions::rewr_221_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 222 + 3] = rewr_functions::rewr_222_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 222 + 3] = rewr_functions::rewr_222_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 222 + 6] = rewr_functions::rewr_222_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 222 + 6] = rewr_functions::rewr_222_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 227 + 2] = rewr_functions::rewr_227_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 227 + 2] = rewr_functions::rewr_227_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 228 + 2] = rewr_functions::rewr_228_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 228 + 2] = rewr_functions::rewr_228_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 229 + 3] = rewr_functions::rewr_229_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 229 + 3] = rewr_functions::rewr_229_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 229 + 6] = rewr_functions::rewr_229_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 229 + 6] = rewr_functions::rewr_229_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 234 + 2] = rewr_functions::rewr_234_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 234 + 2] = rewr_functions::rewr_234_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 235 + 2] = rewr_functions::rewr_235_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 235 + 2] = rewr_functions::rewr_235_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 236 + 3] = rewr_functions::rewr_236_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 236 + 3] = rewr_functions::rewr_236_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 236 + 7] = rewr_functions::rewr_236_7_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 236 + 7] = rewr_functions::rewr_236_7_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 241 + 2] = rewr_functions::rewr_241_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 241 + 2] = rewr_functions::rewr_241_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 242 + 2] = rewr_functions::rewr_242_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 242 + 2] = rewr_functions::rewr_242_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 243 + 3] = rewr_functions::rewr_243_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 243 + 3] = rewr_functions::rewr_243_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 243 + 5] = rewr_functions::rewr_243_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 243 + 5] = rewr_functions::rewr_243_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 252 + 2] = rewr_functions::rewr_252_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 252 + 2] = rewr_functions::rewr_252_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 253 + 2] = rewr_functions::rewr_253_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 253 + 2] = rewr_functions::rewr_253_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 254 + 3] = rewr_functions::rewr_254_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 254 + 3] = rewr_functions::rewr_254_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 254 + 4] = rewr_functions::rewr_254_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 254 + 4] = rewr_functions::rewr_254_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 259 + 2] = rewr_functions::rewr_259_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 259 + 2] = rewr_functions::rewr_259_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 260 + 2] = rewr_functions::rewr_260_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 260 + 2] = rewr_functions::rewr_260_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 261 + 3] = rewr_functions::rewr_261_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 261 + 3] = rewr_functions::rewr_261_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 261 + 5] = rewr_functions::rewr_261_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 261 + 5] = rewr_functions::rewr_261_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 270 + 2] = rewr_functions::rewr_270_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 270 + 2] = rewr_functions::rewr_270_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 271 + 2] = rewr_functions::rewr_271_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 271 + 2] = rewr_functions::rewr_271_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 272 + 3] = rewr_functions::rewr_272_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 272 + 3] = rewr_functions::rewr_272_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 272 + 4] = rewr_functions::rewr_272_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 272 + 4] = rewr_functions::rewr_272_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 277 + 2] = rewr_functions::rewr_277_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 277 + 2] = rewr_functions::rewr_277_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 278 + 2] = rewr_functions::rewr_278_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 278 + 2] = rewr_functions::rewr_278_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 279 + 3] = rewr_functions::rewr_279_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 279 + 3] = rewr_functions::rewr_279_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 279 + 5] = rewr_functions::rewr_279_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 279 + 5] = rewr_functions::rewr_279_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 284 + 2] = rewr_functions::rewr_284_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 284 + 2] = rewr_functions::rewr_284_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 285 + 2] = rewr_functions::rewr_285_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 285 + 2] = rewr_functions::rewr_285_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 286 + 3] = rewr_functions::rewr_286_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 286 + 3] = rewr_functions::rewr_286_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 286 + 5] = rewr_functions::rewr_286_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 286 + 5] = rewr_functions::rewr_286_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 295 + 2] = rewr_functions::rewr_295_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 295 + 2] = rewr_functions::rewr_295_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 296 + 2] = rewr_functions::rewr_296_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 296 + 2] = rewr_functions::rewr_296_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 297 + 3] = rewr_functions::rewr_297_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 297 + 3] = rewr_functions::rewr_297_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 297 + 4] = rewr_functions::rewr_297_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 297 + 4] = rewr_functions::rewr_297_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 302 + 2] = rewr_functions::rewr_302_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 302 + 2] = rewr_functions::rewr_302_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 303 + 2] = rewr_functions::rewr_303_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 303 + 2] = rewr_functions::rewr_303_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 304 + 3] = rewr_functions::rewr_304_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 304 + 3] = rewr_functions::rewr_304_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 304 + 5] = rewr_functions::rewr_304_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 304 + 5] = rewr_functions::rewr_304_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 313 + 2] = rewr_functions::rewr_313_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 313 + 2] = rewr_functions::rewr_313_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 314 + 2] = rewr_functions::rewr_314_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 314 + 2] = rewr_functions::rewr_314_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 315 + 3] = rewr_functions::rewr_315_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 315 + 3] = rewr_functions::rewr_315_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 315 + 4] = rewr_functions::rewr_315_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 315 + 4] = rewr_functions::rewr_315_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 320 + 2] = rewr_functions::rewr_320_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 320 + 2] = rewr_functions::rewr_320_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 321 + 2] = rewr_functions::rewr_321_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 321 + 2] = rewr_functions::rewr_321_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 322 + 3] = rewr_functions::rewr_322_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 322 + 3] = rewr_functions::rewr_322_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 322 + 5] = rewr_functions::rewr_322_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 322 + 5] = rewr_functions::rewr_322_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 331 + 2] = rewr_functions::rewr_331_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 331 + 2] = rewr_functions::rewr_331_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 332 + 2] = rewr_functions::rewr_332_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 332 + 2] = rewr_functions::rewr_332_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 333 + 3] = rewr_functions::rewr_333_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 333 + 3] = rewr_functions::rewr_333_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 333 + 4] = rewr_functions::rewr_333_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 333 + 4] = rewr_functions::rewr_333_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 342 + 2] = rewr_functions::rewr_342_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 342 + 2] = rewr_functions::rewr_342_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 343 + 2] = rewr_functions::rewr_343_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 343 + 2] = rewr_functions::rewr_343_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 344 + 3] = rewr_functions::rewr_344_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 344 + 3] = rewr_functions::rewr_344_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 344 + 4] = rewr_functions::rewr_344_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 344 + 4] = rewr_functions::rewr_344_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 349 + 2] = rewr_functions::rewr_349_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 349 + 2] = rewr_functions::rewr_349_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 350 + 2] = rewr_functions::rewr_350_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 350 + 2] = rewr_functions::rewr_350_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 351 + 3] = rewr_functions::rewr_351_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 351 + 3] = rewr_functions::rewr_351_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 351 + 5] = rewr_functions::rewr_351_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 351 + 5] = rewr_functions::rewr_351_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 356 + 2] = rewr_functions::rewr_356_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 356 + 2] = rewr_functions::rewr_356_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 357 + 2] = rewr_functions::rewr_357_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 357 + 2] = rewr_functions::rewr_357_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 358 + 3] = rewr_functions::rewr_358_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 358 + 3] = rewr_functions::rewr_358_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 358 + 5] = rewr_functions::rewr_358_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 358 + 5] = rewr_functions::rewr_358_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 363 + 2] = rewr_functions::rewr_363_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 363 + 2] = rewr_functions::rewr_363_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 364 + 2] = rewr_functions::rewr_364_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 364 + 2] = rewr_functions::rewr_364_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 365 + 3] = rewr_functions::rewr_365_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 365 + 3] = rewr_functions::rewr_365_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 365 + 6] = rewr_functions::rewr_365_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 365 + 6] = rewr_functions::rewr_365_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 370 + 2] = rewr_functions::rewr_370_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 370 + 2] = rewr_functions::rewr_370_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 371 + 2] = rewr_functions::rewr_371_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 371 + 2] = rewr_functions::rewr_371_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 372 + 3] = rewr_functions::rewr_372_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 372 + 3] = rewr_functions::rewr_372_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 372 + 5] = rewr_functions::rewr_372_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 372 + 5] = rewr_functions::rewr_372_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 377 + 2] = rewr_functions::rewr_377_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 377 + 2] = rewr_functions::rewr_377_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 378 + 2] = rewr_functions::rewr_378_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 378 + 2] = rewr_functions::rewr_378_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 379 + 3] = rewr_functions::rewr_379_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 379 + 3] = rewr_functions::rewr_379_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 379 + 8] = rewr_functions::rewr_379_8_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 379 + 8] = rewr_functions::rewr_379_8_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 384 + 2] = rewr_functions::rewr_384_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 384 + 2] = rewr_functions::rewr_384_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 385 + 2] = rewr_functions::rewr_385_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 385 + 2] = rewr_functions::rewr_385_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 386 + 3] = rewr_functions::rewr_386_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 386 + 3] = rewr_functions::rewr_386_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 386 + 8] = rewr_functions::rewr_386_8_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 386 + 8] = rewr_functions::rewr_386_8_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 391 + 2] = rewr_functions::rewr_391_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 391 + 2] = rewr_functions::rewr_391_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 392 + 2] = rewr_functions::rewr_392_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 392 + 2] = rewr_functions::rewr_392_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 393 + 3] = rewr_functions::rewr_393_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 393 + 3] = rewr_functions::rewr_393_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 393 + 5] = rewr_functions::rewr_393_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 393 + 5] = rewr_functions::rewr_393_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 398 + 2] = rewr_functions::rewr_398_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 398 + 2] = rewr_functions::rewr_398_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 399 + 2] = rewr_functions::rewr_399_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 399 + 2] = rewr_functions::rewr_399_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 400 + 3] = rewr_functions::rewr_400_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 400 + 3] = rewr_functions::rewr_400_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 400 + 5] = rewr_functions::rewr_400_5_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 400 + 5] = rewr_functions::rewr_400_5_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 405 + 2] = rewr_functions::rewr_405_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 405 + 2] = rewr_functions::rewr_405_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 406 + 2] = rewr_functions::rewr_406_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 406 + 2] = rewr_functions::rewr_406_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 407 + 3] = rewr_functions::rewr_407_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 407 + 3] = rewr_functions::rewr_407_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 407 + 6] = rewr_functions::rewr_407_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 407 + 6] = rewr_functions::rewr_407_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 412 + 2] = rewr_functions::rewr_412_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 412 + 2] = rewr_functions::rewr_412_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 413 + 2] = rewr_functions::rewr_413_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 413 + 2] = rewr_functions::rewr_413_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 414 + 3] = rewr_functions::rewr_414_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 414 + 3] = rewr_functions::rewr_414_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 414 + 6] = rewr_functions::rewr_414_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 414 + 6] = rewr_functions::rewr_414_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 419 + 2] = rewr_functions::rewr_419_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 419 + 2] = rewr_functions::rewr_419_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 420 + 2] = rewr_functions::rewr_420_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 420 + 2] = rewr_functions::rewr_420_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 421 + 3] = rewr_functions::rewr_421_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 421 + 3] = rewr_functions::rewr_421_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 421 + 6] = rewr_functions::rewr_421_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 421 + 6] = rewr_functions::rewr_421_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 426 + 2] = rewr_functions::rewr_426_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 426 + 2] = rewr_functions::rewr_426_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 427 + 2] = rewr_functions::rewr_427_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 427 + 2] = rewr_functions::rewr_427_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 428 + 3] = rewr_functions::rewr_428_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 428 + 3] = rewr_functions::rewr_428_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 428 + 6] = rewr_functions::rewr_428_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 428 + 6] = rewr_functions::rewr_428_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 433 + 2] = rewr_functions::rewr_433_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 433 + 2] = rewr_functions::rewr_433_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 434 + 2] = rewr_functions::rewr_434_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 434 + 2] = rewr_functions::rewr_434_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 435 + 3] = rewr_functions::rewr_435_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 435 + 3] = rewr_functions::rewr_435_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 435 + 6] = rewr_functions::rewr_435_6_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 435 + 6] = rewr_functions::rewr_435_6_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 444 + 2] = rewr_functions::rewr_444_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 444 + 2] = rewr_functions::rewr_444_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 445 + 2] = rewr_functions::rewr_445_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 445 + 2] = rewr_functions::rewr_445_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 446 + 3] = rewr_functions::rewr_446_3_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 446 + 3] = rewr_functions::rewr_446_3_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 446 + 4] = rewr_functions::rewr_446_4_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 446 + 4] = rewr_functions::rewr_446_4_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 462 + 1] = rewr_functions::rewr_462_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 462 + 1] = rewr_functions::rewr_462_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 463 + 2] = rewr_functions::rewr_463_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 463 + 2] = rewr_functions::rewr_463_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 477 + 1] = rewr_functions::rewr_477_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 477 + 1] = rewr_functions::rewr_477_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 478 + 2] = rewr_functions::rewr_478_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 478 + 2] = rewr_functions::rewr_478_2_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 492 + 1] = rewr_functions::rewr_492_1_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 492 + 1] = rewr_functions::rewr_492_1_term_arg_in_normal_form;
  this_rewriter->functions_when_arguments_are_not_in_normal_form[this_rewriter->arity_bound * 493 + 2] = rewr_functions::rewr_493_2_term;
  this_rewriter->functions_when_arguments_are_in_normal_form[this_rewriter->arity_bound * 493 + 2] = rewr_functions::rewr_493_2_term_arg_in_normal_form;
}
