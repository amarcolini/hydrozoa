// Versatile string -*- C++ -*-

// Copyright (C) 2005-2020 Free Software Foundation, Inc.
//
// This file is part of the GNU ISO C++ Library.  This library is free
// software; you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the
// Free Software Foundation; either version 3, or (at your option)
// any later version.

// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// Under Section 7 of GPL version 3, you are granted additional
// permissions described in the GCC Runtime Library Exception, version
// 3.1, as published by the Free Software Foundation.

// You should have received a copy of the GNU General Public License and
// a copy of the GCC Runtime Library Exception along with this program;
// see the files COPYING3 and COPYING.RUNTIME respectively.  If not, see
// <http://www.gnu.org/licenses/>.

/** @file ext/vstring.tcc
 *  This is an internal header file, included by other library headers.
 *  Do not attempt to use it directly. @headername{ext/vstring.h}
 */

#ifndef _VSTRING_TCC
#define _VSTRING_TCC 1

#pragma GCC system_header

#include <bits/cxxabi_forced.h>

namespace __gnu_cxx _GLIBCXX_VISIBILITY(default)
{
_GLIBCXX_BEGIN_NAMESPACE_VERSION

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    const typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::npos;

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    void
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    resize(size_type __n, _CharT __c)
    {
      const size_type __size = this->size();
      if (__size < __n)
	this->append(__n - __size, __c);
      else if (__n < __size)
	this->_M_erase(__n, __size - __n);
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>&
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    _M_append(const _CharT* __s, size_type __n)
    {
      const size_type __len = __n + this->size();

      if (__len <= this->capacity() && !this->_M_is_shared())
	{
	  if (__n)
	    this->_S_copy(this->_M_data() + this->size(), __s, __n);
	}
      else
	this->_M_mutate(this->size(), size_type(0), __s, __n);

      this->_M_set_length(__len);
      return *this;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    template<typename _InputIterator>
      __versa_string<_CharT, _Traits, _Alloc, _Base>&
      __versa_string<_CharT, _Traits, _Alloc, _Base>::
      _M_replace_dispatch(const_iterator __i1, const_iterator __i2,
			  _InputIterator __k1, _InputIterator __k2,
			  std::__false_type)
      {
	const __versa_string __s(__k1, __k2);
	const size_type __n1 = __i2 - __i1;
	return _M_replace(__i1 - _M_ibegin(), __n1, __s._M_data(),
			  __s.size());
      }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>&
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    _M_replace_aux(size_type __pos1, size_type __n1, size_type __n2,
		   _CharT __c)
    {
      _M_check_length(__n1, __n2, "__versa_string::_M_replace_aux");

      const size_type __old_size = this->size();
      const size_type __new_size = __old_size + __n2 - __n1;

      if (__new_size <= this->capacity() && !this->_M_is_shared())
	{
	  _CharT* __p = this->_M_data() + __pos1;

	  const size_type __how_much = __old_size - __pos1 - __n1;
	  if (__how_much && __n1 != __n2)
	    this->_S_move(__p + __n2, __p + __n1, __how_much);
	}
      else
	this->_M_mutate(__pos1, __n1, 0, __n2);

      if (__n2)
	this->_S_assign(this->_M_data() + __pos1, __n2, __c);

      this->_M_set_length(__new_size);
      return *this;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>&
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    _M_replace(size_type __pos, size_type __len1, const _CharT* __s,
	       const size_type __len2)
    {
      _M_check_length(__len1, __len2, "__versa_string::_M_replace");

      const size_type __old_size = this->size();
      const size_type __new_size = __old_size + __len2 - __len1;

      if (__new_size <= this->capacity() && !this->_M_is_shared())
	{
	  _CharT* __p = this->_M_data() + __pos;

	  const size_type __how_much = __old_size - __pos - __len1;
	  if (_M_disjunct(__s))
	    {
	      if (__how_much && __len1 != __len2)
		this->_S_move(__p + __len2, __p + __len1, __how_much);
	      if (__len2)
		this->_S_copy(__p, __s, __len2);
	    }
	  else
	    {
	      // Work in-place.
	      if (__len2 && __len2 <= __len1)
		this->_S_move(__p, __s, __len2);
	      if (__how_much && __len1 != __len2)
		this->_S_move(__p + __len2, __p + __len1, __how_much);
	      if (__len2 > __len1)
		{
		  if (__s + __len2 <= __p + __len1)
		    this->_S_move(__p, __s, __len2);
		  else if (__s >= __p + __len1)
		    this->_S_copy(__p, __s + __len2 - __len1, __len2);
		  else
		    {
		      const size_type __nleft = (__p + __len1) - __s;
		      this->_S_move(__p, __s, __nleft);
		      this->_S_copy(__p + __nleft, __p + __len2,
				    __len2 - __nleft);
		    }
		}
	    }
	}
      else
	this->_M_mutate(__pos, __len1, __s, __len2);

      this->_M_set_length(__new_size);
      return *this;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>
    operator+(const __versa_string<_CharT, _Traits, _Alloc, _Base>& __lhs,
	      const __versa_string<_CharT, _Traits, _Alloc, _Base>& __rhs)
    {
      __versa_string<_CharT, _Traits, _Alloc, _Base> __str;
      __str.reserve(__lhs.size() + __rhs.size());
      __str.append(__lhs);
      __str.append(__rhs);
      return __str;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>
    operator+(const _CharT* __lhs,
	      const __versa_string<_CharT, _Traits, _Alloc, _Base>& __rhs)
    {
      __glibcxx_requires_string(__lhs);
      typedef __versa_string<_CharT, _Traits, _Alloc, _Base> __string_type;
      typedef typename __string_type::size_type	  __size_type;
      const __size_type __len = _Traits::length(__lhs);
      __string_type __str;
      __str.reserve(__len + __rhs.size());
      __str.append(__lhs, __len);
      __str.append(__rhs);
      return __str;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>
    operator+(_CharT __lhs,
	      const __versa_string<_CharT, _Traits, _Alloc, _Base>& __rhs)
    {
      __versa_string<_CharT, _Traits, _Alloc, _Base> __str;
      __str.reserve(__rhs.size() + 1);
      __str.push_back(__lhs);
      __str.append(__rhs);
      return __str;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>
    operator+(const __versa_string<_CharT, _Traits, _Alloc, _Base>& __lhs,
	      const _CharT* __rhs)
    {
      __glibcxx_requires_string(__rhs);
      typedef __versa_string<_CharT, _Traits, _Alloc, _Base> __string_type;
      typedef typename __string_type::size_type	  __size_type;
      const __size_type __len = _Traits::length(__rhs);
      __string_type __str;
      __str.reserve(__lhs.size() + __len);
      __str.append(__lhs);
      __str.append(__rhs, __len);
      return __str;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    __versa_string<_CharT, _Traits, _Alloc, _Base>
    operator+(const __versa_string<_CharT, _Traits, _Alloc, _Base>& __lhs,
	      _CharT __rhs)
    {
      __versa_string<_CharT, _Traits, _Alloc, _Base> __str;
      __str.reserve(__lhs.size() + 1);
      __str.append(__lhs);
      __str.push_back(__rhs);
      return __str;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    copy(_CharT* __s, size_type __n, size_type __pos) const
    {
      _M_check(__pos, "__versa_string::copy");
      __n = _M_limit(__pos, __n);
      __glibcxx_requires_string_len(__s, __n);
      if (__n)
	this->_S_copy(__s, this->_M_data() + __pos, __n);
      // 21.3.5.7 par 3: do not append null.  (good.)
      return __n;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      const size_type __size = this->size();
      const _CharT* __data = this->_M_data();

      if (__n == 0)
	return __pos <= __size ? __pos : npos;

      if (__n <= __size)
	{
	  for (; __pos <= __size - __n; ++__pos)
	    if (traits_type::eq(__data[__pos], __s[0])
		&& traits_type::compare(__data + __pos + 1,
					__s + 1, __n - 1) == 0)
	      return __pos;
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find(_CharT __c, size_type __pos) const _GLIBCXX_NOEXCEPT
    {
      size_type __ret = npos;
      const size_type __size = this->size();
      if (__pos < __size)
	{
	  const _CharT* __data = this->_M_data();
	  const size_type __n = __size - __pos;
	  const _CharT* __p = traits_type::find(__data + __pos, __n, __c);
	  if (__p)
	    __ret = __p - __data;
	}
      return __ret;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    rfind(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      const size_type __size = this->size();
      if (__n <= __size)
	{
	  __pos = std::min(size_type(__size - __n), __pos);
	  const _CharT* __data = this->_M_data();
	  do
	    {
	      if (traits_type::compare(__data + __pos, __s, __n) == 0)
		return __pos;
	    }
	  while (__pos-- > 0);
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    rfind(_CharT __c, size_type __pos) const _GLIBCXX_NOEXCEPT
    {
      size_type __size = this->size();
      if (__size)
	{
	  if (--__size > __pos)
	    __size = __pos;
	  for (++__size; __size-- > 0; )
	    if (traits_type::eq(this->_M_data()[__size], __c))
	      return __size;
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_first_of(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      for (; __n && __pos < this->size(); ++__pos)
	{
	  const _CharT* __p = traits_type::find(__s, __n,
						this->_M_data()[__pos]);
	  if (__p)
	    return __pos;
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_last_of(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      size_type __size = this->size();
      if (__size && __n)
	{
	  if (--__size > __pos)
	    __size = __pos;
	  do
	    {
	      if (traits_type::find(__s, __n, this->_M_data()[__size]))
		return __size;
	    }
	  while (__size-- != 0);
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_first_not_of(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      for (; __pos < this->size(); ++__pos)
	if (!traits_type::find(__s, __n, this->_M_data()[__pos]))
	  return __pos;
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_first_not_of(_CharT __c, size_type __pos) const _GLIBCXX_NOEXCEPT
    {
      for (; __pos < this->size(); ++__pos)
	if (!traits_type::eq(this->_M_data()[__pos], __c))
	  return __pos;
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_last_not_of(const _CharT* __s, size_type __pos, size_type __n) const
    {
      __glibcxx_requires_string_len(__s, __n);
      size_type __size = this->size();
      if (__size)
	{
	  if (--__size > __pos)
	    __size = __pos;
	  do
	    {
	      if (!traits_type::find(__s, __n, this->_M_data()[__size]))
		return __size;
	    }
	  while (__size--);
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    typename __versa_string<_CharT, _Traits, _Alloc, _Base>::size_type
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    find_last_not_of(_CharT __c, size_type __pos) const _GLIBCXX_NOEXCEPT
    {
      size_type __size = this->size();
      if (__size)
	{
	  if (--__size > __pos)
	    __size = __pos;
	  do
	    {
	      if (!traits_type::eq(this->_M_data()[__size], __c))
		return __size;
	    }
	  while (__size--);
	}
      return npos;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    int
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    compare(size_type __pos, size_type __n, const __versa_string& __str) const
    {
      _M_check(__pos, "__versa_string::compare");
      __n = _M_limit(__pos, __n);
      const size_type __osize = __str.size();
      const size_type __len = std::min(__n, __osize);
      int __r = traits_type::compare(this->_M_data() + __pos,
				     __str.data(), __len);
      if (!__r)
	__r = this->_S_compare(__n, __osize);
      return __r;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    int
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    compare(size_type __pos1, size_type __n1, const __versa_string& __str,
	    size_type __pos2, size_type __n2) const
    {
      _M_check(__pos1, "__versa_string::compare");
      __str._M_check(__pos2, "__versa_string::compare");
      __n1 = _M_limit(__pos1, __n1);
      __n2 = __str._M_limit(__pos2, __n2);
      const size_type __len = std::min(__n1, __n2);
      int __r = traits_type::compare(this->_M_data() + __pos1,
				     __str.data() + __pos2, __len);
      if (!__r)
	__r = this->_S_compare(__n1, __n2);
      return __r;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    int
    __versa_string<_CharT, _Traits, _Alloc, _Base>::
    compare(const _CharT* __s) const
    {
      __glibcxx_requires_string(__s);
      const size_type __size = this->size();
      const size_type __osize = traits_type::length(__s);
      const size_type __len = std::min(__size, __osize);
      int __r = traits_type::compare(this->_M_data(), __s, __len);
      if (!__r)
	__r = this->_S_compare(__size, __osize);
      return __r;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    int
    __versa_string <_CharT, _Traits, _Alloc, _Base>::
    compare(size_type __pos, size_type __n1, const _CharT* __s) const
    {
      __glibcxx_requires_string(__s);
      _M_check(__pos, "__versa_string::compare");
      __n1 = _M_limit(__pos, __n1);
      const size_type __osize = traits_type::length(__s);
      const size_type __len = std::min(__n1, __osize);
      int __r = traits_type::compare(this->_M_data() + __pos, __s, __len);
      if (!__r)
	__r = this->_S_compare(__n1, __osize);
      return __r;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
	   template <typename, typename, typename> class _Base>
    int
    __versa_string <_CharT, _Traits, _Alloc, _Base>::
    compare(size_type __pos, size_type __n1, const _CharT* __s,
	    size_type __n2) const
    {
      __glibcxx_requires_string_len(__s, __n2);
      _M_check(__pos, "__versa_string::compare");
      __n1 = _M_limit(__pos, __n1);
      const size_type __len = std::min(__n1, __n2);
      int __r = traits_type::compare(this->_M_data() + __pos, __s, __len);
      if (!__r)
	__r = this->_S_compare(__n1, __n2);
      return __r;
    }

_GLIBCXX_END_NAMESPACE_VERSION
} // namespace

namespace std _GLIBCXX_VISIBILITY(default)
{
_GLIBCXX_BEGIN_NAMESPACE_VERSION

  template<typename _CharT, typename _Traits, typename _Alloc,
           template <typename, typename, typename> class _Base>
    basic_istream<_CharT, _Traits>&
    operator>>(basic_istream<_CharT, _Traits>& __in,
	       __gnu_cxx::__versa_string<_CharT, _Traits,
	                                 _Alloc, _Base>& __str)
    {
      typedef basic_istream<_CharT, _Traits>            __istream_type;
      typedef typename __istream_type::ios_base         __ios_base;
      typedef __gnu_cxx::__versa_string<_CharT, _Traits, _Alloc, _Base>
	                                                __string_type;
      typedef typename __istream_type::int_type		__int_type;
      typedef typename __string_type::size_type		__size_type;
      typedef core::ffipe<_CharT>				__core::ffipe_type;
      typedef typename __core::ffipe_type::core::ffipe_base         __core::ffipe_base;

      __size_type __extracted = 0;
      typename __ios_base::iostate __err = __ios_base::goodbit;
      typename __istream_type::sentry __cerb(__in, false);
      if (__cerb)
	{
	  __try
	    {
	      // Avoid reallocation for common case.
	      __str.erase();
	      _CharT __buf[128];
	      __size_type __len = 0;
	      const streamsize __w = __in.width();
	      const __size_type __n = __w > 0 ? static_cast<__size_type>(__w)
		                              : __str.max_size();
	      const __core::ffipe_type& __ct = use_facet<__core::ffipe_type>(__in.getloc());
	      const __int_type __eof = _Traits::eof();
	      __int_type __c = __in.rdbuf()->sgetc();

	      while (__extracted < __n
		     && !_Traits::eq_int_type(__c, __eof)
		     && !__ct.is(__core::ffipe_base::space,
				 _Traits::to_char_type(__c)))
		{
		  if (__len == sizeof(__buf) / sizeof(_CharT))
		    {
		      __str.append(__buf, sizeof(__buf) / sizeof(_CharT));
		      __len = 0;
		    }
		  __buf[__len++] = _Traits::to_char_type(__c);
		  ++__extracted;
		  __c = __in.rdbuf()->snextc();
		}
	      __str.append(__buf, __len);

	      if (_Traits::eq_int_type(__c, __eof))
		__err |= __ios_base::eofbit;
	      __in.width(0);
	    }
	  __catch(__cxxabiv1::__forced_unwind&)
	    {
	      __in._M_setstate(__ios_base::badbit);
	      __throw_exception_again;
	    }
	  __catch(...)
	    {
	      // _GLIBCXX_RESOLVE_LIB_DEFECTS
	      // 91. Description of operator>> and getline() for string<>
	      // might cause endless loop
	      __in._M_setstate(__ios_base::badbit);
	    }
	}
      // 211.  operator>>(istream&, string&) doesn't set failbit
      if (!__extracted)
	__err |= __ios_base::failbit;
      if (__err)
	__in.setstate(__err);
      return __in;
    }

  template<typename _CharT, typename _Traits, typename _Alloc,
           template <typename, typename, typename> class _Base>
    basic_istream<_CharT, _Traits>&
    getline(basic_istream<_CharT, _Traits>& __in,
	    __gnu_cxx::__versa_string<_CharT, _Traits, _Alloc, _Base>& __str,
	    _CharT __delim)
    {
      typedef basic_istream<_CharT, _Traits>	        __istream_type;
      typedef typename __istream_type::ios_base         __ios_base;
      typedef __gnu_cxx::__versa_string<_CharT, _Traits, _Alloc, _Base>
	                                                __string_type;
      typedef typename __istream_type::int_type		__int_type;
      typedef typename __string_type::size_type		__size_type;

      __size_type __extracted = 0;
      const __size_type __n = __str.max_size();
      typename __ios_base::iostate __err = __ios_base::goodbit;
      typename __istream_type::sentry __cerb(__in, true);
      if (__cerb)
	{
	  __try
	    {
	      // Avoid reallocation for common case.
	      __str.erase();
	      _CharT __buf[128];
	      __size_type __len = 0;
	      const __int_type __idelim = _Traits::to_int_type(__delim);
	      const __int_type __eof = _Traits::eof();
	      __int_type __c = __in.rdbuf()->sgetc();

	      while (__extracted < __n
		     && !_Traits::eq_int_type(__c, __eof)
		     && !_Traits::eq_int_type(__c, __idelim))
		{
		  if (__len == sizeof(__buf) / sizeof(_CharT))
		    {
		      __str.append(__buf, sizeof(__buf) / sizeof(_CharT));
		      __len = 0;
		    }
		  __buf[__len++] = _Traits::to_char_type(__c);
		  ++__extracted;
		  __c = __in.rdbuf()->snextc();
		}
	      __str.append(__buf, __len);

	      if (_Traits::eq_int_type(__c, __eof))
		__err |= __ios_base::eofbit;
	      else if (_Traits::eq_int_type(__c, __idelim))
		{
		  ++__extracted;
		  __in.rdbuf()->sbumpc();
		}
	      else
		__err |= __ios_base::failbit;
	    }
	  __catch(__cxxabiv1::__forced_unwind&)
	    {
	      __in._M_setstate(__ios_base::badbit);
	      __throw_exception_again;
	    }
	  __catch(...)
	    {
	      // _GLIBCXX_RESOLVE_LIB_DEFECTS
	      // 91. Description of operator>> and getline() for string<>
	      // might cause endless loop
	      __in._M_setstate(__ios_base::badbit);
	    }
	}
      if (!__extracted)
	__err |= __ios_base::failbit;
      if (__err)
	__in.setstate(__err);
      return __in;
    }

_GLIBCXX_END_NAMESPACE_VERSION
} // namespace

#endif // _VSTRING_TCC
