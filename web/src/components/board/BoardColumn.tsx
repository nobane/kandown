// web/src/components/board/BoardColumn.tsx
import React, {forwardRef, ReactNode} from 'react'
import {UniqueIdentifier} from '@dnd-kit/core'
import {useSortable} from '@dnd-kit/sortable'
import {CSS} from '@dnd-kit/utilities'
import css from './BoardColumn.module.css'
import {Remove, Handle} from 'src/components/card/CardComponents'

interface ColumnProps {
    children: ReactNode
    id: UniqueIdentifier
    items: UniqueIdentifier[]
    columns?: number
    label?: string
    style?: React.CSSProperties
    horizontal?: boolean
    hover?: boolean
    handleProps?: React.HTMLAttributes<any>
    scrollable?: boolean
    shadow?: boolean
    placeholder?: boolean
    unstyled?: boolean
    disabled?: boolean
    onClick?(): void
    onRemove?(): void
}

export const Column = forwardRef<HTMLElement, ColumnProps>(
    (
        {
            children,
            id,
            items,
            handleProps,
            horizontal,
            hover,
            onClick,
            onRemove,
            label,
            placeholder,
            style,
            scrollable,
            shadow,
            unstyled,
            disabled,
            ...props
        },
        ref,
    ) => {
        const {
            attributes,
            isDragging,
            listeners,
            over,
            setNodeRef,
            active,
            transition,
            transform,
        } = useSortable({
            id,
            data: {
                type: 'container',
                children: items,
            },
            disabled,
        })

        const isOverContainer = over
            ? (id === over.id && active?.data.current?.type !== 'container') ||
              items.includes(over.id)
            : false

        // Create component type based on whether it's clickable
        const Component = onClick ? 'button' : 'div'

        // Handle refs properly
        const internalRef = (node: HTMLElement | null) => {
            // Call the sortable ref
            if (setNodeRef) {
                setNodeRef(node)
            }

            // Forward the ref if provided externally
            if (typeof ref === 'function') {
                ref(node)
            } else if (ref) {
                ;(ref as React.MutableRefObject<HTMLElement | null>).current = node
            }
        }

        const classNames = [
            css.container,
            unstyled ? css.unstyled : '',
            horizontal ? css.horizontal : '',
            hover || isOverContainer ? css.hover : '',
            placeholder ? css.placeholder : '',
            scrollable ? css.scrollable : '',
            shadow ? css.shadow : '',
        ]
            .filter(Boolean)
            .join(' ')

        return (
            <Component
                {...props}
                ref={(disabled ? ref : internalRef) as unknown as any}
                style={
                    {
                        ...style,
                        transition,
                        transform: CSS.Translate.toString(transform),
                        opacity: isDragging ? 0.5 : undefined,
                    } as React.CSSProperties
                }
                className={classNames}
                onClick={onClick}
                tabIndex={onClick ? 0 : undefined}
                {...(disabled
                    ? {}
                    : {
                          ...attributes,
                          ...listeners,
                      })}
            >
                {label ? (
                    <div className={css.header}>
                        {label}
                        <div className={css.actions}>
                            {onRemove ? <Remove onClick={onRemove} /> : undefined}
                            {!disabled && <Handle {...handleProps} />}
                        </div>
                    </div>
                ) : null}
                {placeholder ? children : <ul>{children}</ul>}
            </Component>
        )
    },
)
