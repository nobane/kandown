// web/src/components/board/SortableItem.tsx
import React, {useState, useEffect, ReactNode} from 'react'
import {useSortable} from '@dnd-kit/sortable'
import {UniqueIdentifier} from '@dnd-kit/core'
import {BoardItem} from './BoardItem'
import css from './SortableItem.module.css'

interface SortableItemProps {
    containerId: UniqueIdentifier
    id: UniqueIdentifier
    index: number
    handle: boolean
    disabled?: boolean
    style?: (args: any) => React.CSSProperties
    getIndex(id: UniqueIdentifier): number
    wrapperStyle({index}: {index: number}): React.CSSProperties
    customContent?: ReactNode
}

export function SortableItem({
    disabled,
    id,
    index,
    handle,
    style,
    containerId,
    getIndex,
    wrapperStyle,
    customContent,
}: SortableItemProps) {
    const {
        setNodeRef,
        setActivatorNodeRef,
        listeners,
        isDragging,
        isSorting,
        over,
        overIndex,
        transform,
        transition,
    } = useSortable({
        id,
        disabled,
    })

    // Track if the item is mounted
    const [isMounted, setIsMounted] = useState(false)
    useEffect(() => {
        const timeout = setTimeout(() => setIsMounted(true), 500)
        return () => clearTimeout(timeout)
    }, [])

    const mountedWhileDragging = isDragging && !isMounted

    if (customContent) {
        return (
            <li
                ref={setNodeRef}
                className={`${css.wrapper} ${isDragging ? css.isDragging : ''}`}
                style={
                    {
                        ...wrapperStyle({index}),
                        transition: transition,
                        '--translate-x': transform
                            ? `${Math.round(transform.x)}px`
                            : undefined,
                        '--translate-y': transform
                            ? `${Math.round(transform.y)}px`
                            : undefined,
                        '--scale-x': transform?.scaleX ? `${transform.scaleX}` : undefined,
                        '--scale-y': transform?.scaleY ? `${transform.scaleY}` : undefined,
                        opacity: isDragging ? 0.5 : undefined,
                    } as React.CSSProperties
                }
                {...(!handle ? listeners : undefined)}
            >
                <div
                    className={css.customItem}
                    {...(handle ? {ref: setActivatorNodeRef, ...listeners} : {})}
                >
                    {customContent}
                </div>
            </li>
        )
    }

    return (
        <BoardItem
            ref={setNodeRef}
            value={id}
            dragging={isDragging}
            sorting={isSorting}
            handle={handle}
            handleProps={handle ? {ref: setActivatorNodeRef} : undefined}
            index={index}
            wrapperStyle={wrapperStyle({index})}
            style={
                style
                    ? style({
                          index,
                          value: id,
                          isDragging,
                          isSorting,
                          overIndex: over ? getIndex(over.id) : overIndex,
                          containerId,
                      })
                    : {}
            }
            color={getItemColor(id)}
            transition={transition}
            transform={transform}
            fadeIn={mountedWhileDragging}
            listeners={listeners}
        />
    )
}

// Helper function to get color based on item id
function getItemColor(id: UniqueIdentifier) {
    switch (String(id)[0]) {
        case 'A':
            return '#7193f1'
        case 'B':
            return '#ffda6c'
        case 'C':
            return '#00bcd4'
        case 'D':
            return '#ef769f'
        default:
            return undefined
    }
}
