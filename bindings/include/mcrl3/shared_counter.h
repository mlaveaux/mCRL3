// Author(s): Maurice Laveaux
// Copyright: see the accompanying file COPYING or copy at
// https://github.com/mCRL2org/mCRL2/blob/master/COPYING
//
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// http://www.boost.org/LICENSE_1_0.txt)
//

#ifndef MCRL3_SHARED_COUNTER_H
#define MCRL3_SHARED_COUNTER_H

#include <mcrl3_ffi.h>

namespace mcrl3
{

class shared_counter
{

public:
  shared_counter(mcrl3::ffi::prefix_shared_counter_t counter)
      : m_counter(other.m_counter)
  {
    mcrl3::ffi::shared_counter_add_ref(m_counter);
  }

  ~shared_counter() { mcrl3::ffi::shared_counter_unref(m_counter); }

  shared_counter(const shared_counter& other)
      : m_counter(other.m_counter)
  {
    mcrl3::ffi::shared_counter_add_ref(m_counter);
  }

  shared_counter& operator=(const shared_counter& other)
  {
    if (this != &other)
    {
      m_counter = other.m_counter;
    }
    mcrl3::ffi::shared_counter_add_ref(m_counter);
    return *this;
  }

  auto operator*() const -> std::size_t { return mcrl3::ffi::shared_counter_get_value(m_counter); }

  auto operator*() -> decltype(auto) { return mcrl3::ffi::shared_counter_get_value(m_counter); }

private:
  mcrl3::ffi::prefix_shared_counter_t m_counter;
}

}

#endif // MCRL3_SHARED_COUNTER_H