// Platform detection utilities

const UA = typeof navigator !== 'undefined' ? navigator.userAgent : ''

export const isAndroid = /Android/i.test(UA)
export const isIOS = /iPhone|iPad|iPod/i.test(UA)
export const isMobile = isAndroid || isIOS
export const isDesktop = !isMobile

// Android native bridges
export const hasNativeFileResolver = typeof window !== 'undefined' && !!window.NativeFileResolver
export const hasNativeTts = typeof window !== 'undefined' && !!window.NativeTts && window.NativeTts.isAvailable && window.NativeTts.isAvailable()
