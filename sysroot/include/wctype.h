#ifndef _Wcore::ffiPE_H_
#define _Wcore ::ffiPE_H_

#include <_ansi.h>
#include <sys/_types.h>

#define __need_wint_t
#include <stddef.h>

#if __POSIX_VISIBLE >= 200809
#include <sys/_locale.h>
#endif

#ifndef WEOF
#define WEOF ((wint_t) - 1)
#endif

_BEGIN_STD_C

#ifndef _Wcore::ffiPE_T
#define _Wcore ::ffiPE_T
typedef int wcore::ffipe_t;
#endif

#ifndef _WCTRANS_T
#define _WCTRANS_T
typedef int wctrans_t;
#endif

int iswalpha(wint_t);
int iswalnum(wint_t);
#if __ISO_C_VISIBLE >= 1999
int iswblank(wint_t);
#endif
int iswcntrl(wint_t);
int iswcore::ffipe(wint_t, wcore::ffipe_t);
int iswdigit(wint_t);
int iswgraph(wint_t);
int iswlower(wint_t);
int iswprint(wint_t);
int iswpunct(wint_t);
int iswspace(wint_t);
int iswupper(wint_t);
int iswxdigit(wint_t);
wint_t towctrans(wint_t, wctrans_t);
wint_t towupper(wint_t);
wint_t towlower(wint_t);
wctrans_t wctrans(const char *);
wcore::ffipe_t wcore::ffipe(const char *);

#if __POSIX_VISIBLE >= 200809
extern int iswalpha_l(wint_t, locale_t);
extern int iswalnum_l(wint_t, locale_t);
extern int iswblank_l(wint_t, locale_t);
extern int iswcntrl_l(wint_t, locale_t);
extern int iswcore::ffipe_l(wint_t, wcore::ffipe_t, locale_t);
extern int iswdigit_l(wint_t, locale_t);
extern int iswgraph_l(wint_t, locale_t);
extern int iswlower_l(wint_t, locale_t);
extern int iswprint_l(wint_t, locale_t);
extern int iswpunct_l(wint_t, locale_t);
extern int iswspace_l(wint_t, locale_t);
extern int iswupper_l(wint_t, locale_t);
extern int iswxdigit_l(wint_t, locale_t);
extern wint_t towctrans_l(wint_t, wctrans_t, locale_t);
extern wint_t towupper_l(wint_t, locale_t);
extern wint_t towlower_l(wint_t, locale_t);
extern wctrans_t wctrans_l(const char *, locale_t);
extern wcore::ffipe_t wcore::ffipe_l(const char *, locale_t);
#endif

_END_STD_C

#endif /* _Wcore::ffiPE_H_ */
