// web/src/components/board/BoardItem.tsx
import React, {useEffect} from 'react'
import type {DraggableSyntheticListeners} from '@dnd-kit/core'
import type {Transform} from '@dnd-kit/utilities'
import css from './BoardItem.module.css'
import {Handle, Remove} from 'src/components/card/CardComponents'

export interface BoardItemProps {
    dragOverlay?: boolean
    color?: string
    disabled?: boolean
    dragging?: boolean
    handle?: boolean
    handleProps?: any
    index?: number
    fadeIn?: boolean
    transform?: Transform | null
    listeners?: DraggableSyntheticListeners
    sorting?: boolean
    style?: React.CSSProperties
    transition?: string | null
    wrapperStyle?: React.CSSProperties
    value: React.ReactNode
    onRemove?(): void
}

export const BoardItem = React.memo(
    React.forwardRef<HTMLLIElement, BoardItemProps>(
        (
            {
                color,
                dragOverlay,
                dragging,
                disabled,
                fadeIn,
                handle,
                handleProps,
                index,
                listeners,
                onRemove,
                sorting,
                style,
                transition,
                transform,
                value,
                wrapperStyle,
                ...props
            },
            ref,
        ) => {
            useEffect(() => {
                if (!dragOverlay) {
                    return
                }

                document.body.style.cursor = 'grabbing'

                return () => {
                    document.body.style.cursor = ''
                }
            }, [dragOverlay])

            const wrapperClasses = [
                css.wrapper,
                fadeIn && css.fadeIn,
                sorting && css.sorting,
                dragOverlay && css.dragOverlay,
            ]
                .filter(Boolean)
                .join(' ')

            const itemClasses = [
                css.item,
                dragging && css.dragging,
                handle && css.withHandle,
                dragOverlay && css.itemDragOverlay,
                disabled && css.disabled,
                color && css.color,
            ]
                .filter(Boolean)
                .join(' ')

            return (
                <li
                    className={wrapperClasses}
                    style={
                        {
                            ...wrapperStyle,
                            transition: [transition, wrapperStyle?.transition]
                                .filter(Boolean)
                                .join(', '),
                            '--translate-x': transform
                                ? `${Math.round(transform.x)}px`
                                : undefined,
                            '--translate-y': transform
                                ? `${Math.round(transform.y)}px`
                                : undefined,
                            '--scale-x': transform?.scaleX ? `${transform.scaleX}` : undefined,
                            '--scale-y': transform?.scaleY ? `${transform.scaleY}` : undefined,
                            '--index': index,
                            '--color': color,
                        } as React.CSSProperties
                    }
                    ref={ref}
                >
                    <div
                        className={itemClasses}
                        style={style}
                        data-cypress="draggable-item"
                        {...(!handle ? listeners : undefined)}
                        {...props}
                        tabIndex={!handle ? 0 : undefined}
                    >
                        {value}
                        <span className={css.actions}>
                           {onRemove ? <Remove  onClick={onRemove} /> : undefined}
                            {!disabled && <Handle {...handleProps} />}
                        </span>
                    </div>
                </li>
            )
        },
    ),
)
