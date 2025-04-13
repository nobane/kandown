// web/src/components/card/CardComponents.tsx
import React from 'react'
import css from './CardComponents.module.css'


export function Handle(props: React.HTMLAttributes<HTMLButtonElement>) {
    return (
        <button {...props} className={css.handle} tabIndex={0}>
            <svg viewBox="0 0 20 20" width="12">
                <path d="M7 2a2 2 0 1 0 .001 4.001A2 2 0 0 0 7 2zm0 6a2 2 0 1 0 .001 4.001A2 2 0 0 0 7 8zm0 6a2 2 0 1 0 .001 4.001A2 2 0 0 0 7 14zm6-8a2 2 0 1 0-.001-4.001A2 2 0 0 0 13 6zm0 2a2 2 0 1 0 .001 4.001A2 2 0 0 0 13 8zm0 6a2 2 0 1 0 .001 4.001A2 2 0 0 0 13 14z"></path>
            </svg>
        </button>
    )
}

interface RemoveProps extends React.HTMLAttributes<HTMLButtonElement> {
    className?: string
}

export function Remove({className, ...props}: RemoveProps) {
    return (
        <button {...props} className={`${css.remove} ${className || ''}`} tabIndex={0}>
            <svg viewBox="0 0 22 22" width="8">
                <path d="M3.42 2L21 19.57l-1.41 1.42L19.57 21l-7.99-8.01-9.19 9.19-1.41-1.41 9.19-9.19L1.42 2.83 2.83 1.41 12.01 10.6 19.58 3 3.42 2z" />
            </svg>
        </button>
    )
}
