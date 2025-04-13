// web/src/components/board/KanbanBoard.tsx
import React, {useCallback, useEffect, useRef, useState, ReactNode} from 'react'
import {createPortal} from 'react-dom'
import {
    DndContext,
    DragOverlay,
    closestCenter,
    pointerWithin,
    rectIntersection,
    getFirstCollision,
    KeyboardSensor,
    MouseSensor,
    TouchSensor,
    useSensors,
    useSensor,
    MeasuringStrategy,
    UniqueIdentifier,
    CollisionDetection,
    DropAnimation,
    defaultDropAnimationSideEffects,
    CancelDrop,
    Modifiers,
    KeyboardCoordinateGetter,
} from '@dnd-kit/core'
import {
    SortableContext,
    verticalListSortingStrategy,
    horizontalListSortingStrategy,
    arrayMove,
    SortingStrategy,
} from '@dnd-kit/sortable'

import {Column} from './BoardColumn'
import {BoardItem} from './BoardItem'
import {SortableItem} from './SortableItem'
import css from './KanbanBoard.module.css'
import {keyboardCoordGetter} from 'src/utils/keyboardCoordinates'
import {createRange} from 'src/utils/helpers'

// Constants
const PLACEHOLDER_ID = 'placeholder'

// Types
export type Items = Record<UniqueIdentifier, UniqueIdentifier[]>

interface KanbanBoardProps {
    itemCount?: number
    initialItems?: Items
    strategy?: SortingStrategy
    vertical?: boolean
    handle?: boolean
    cancelDrop?: CancelDrop
    coordinateGetter?: KeyboardCoordinateGetter
    getItemStyles?: (args: any) => React.CSSProperties
    wrapperStyle?: (args: {index: number}) => React.CSSProperties
    modifiers?: Modifiers
    containerStyle?: React.CSSProperties
    adjustScale?: boolean
    renderItem?: (id: UniqueIdentifier) => ReactNode
    renderContainerHeader?: (containerId: UniqueIdentifier) => ReactNode
    onItemsChange?: (items: Items) => void
}

export function KanbanBoard({
    itemCount = 3,
    initialItems,
    strategy = verticalListSortingStrategy,
    vertical = false,
    handle = false,
    cancelDrop,
    coordinateGetter = keyboardCoordGetter,
    getItemStyles = () => ({}),
    wrapperStyle = () => ({}),
    modifiers,
    containerStyle,
    adjustScale = false,
    renderItem,
    renderContainerHeader,
    onItemsChange,
}: KanbanBoardProps) {
    // State
    const [items, setItems] = useState<Items>(() => {
        console.debug('[KanbanBoard] Initializing with items:', initialItems)

        if (initialItems && Object.keys(initialItems).length > 0) {
            // Convert any non-array values to arrays
            const processedItems: Items = {}
            Object.entries(initialItems).forEach(([key, value]) => {
                processedItems[key] = Array.isArray(value) ? value : [value]
            })
            return processedItems
        }

        // Default items if none provided
        console.debug('[KanbanBoard] No valid initialItems, using defaults')
        return {
            A: createRange(itemCount, index => `A${index + 1}`),
            B: createRange(itemCount, index => `B${index + 1}`),
            C: createRange(itemCount, index => `C${index + 1}`),
        }
    })

    const [containers, setContainers] = useState(Object.keys(items) as UniqueIdentifier[])
    const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null)
    const [clonedItems, setClonedItems] = useState<Items | null>(null)

    useEffect(() => {
        console.debug('[KanbanBoard] Running with items:', items)
        console.debug('[KanbanBoard] Containers:', containers)
    }, [items, containers])
    // Refs
    const lastOverId = useRef<UniqueIdentifier | null>(null)
    const recentlyMovedToNewContainer = useRef(false)

    // Setup sensors
    const sensors = useSensors(
        useSensor(MouseSensor),
        useSensor(TouchSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter,
        }),
    )

    const isSortingContainer = activeId ? containers.includes(activeId) : false

    // Update parent component when items change
    useEffect(() => {
        if (onItemsChange && initialItems) {
            onItemsChange(items)
        }
    }, [items, onItemsChange, initialItems])

    // Helper functions
    const findContainer = useCallback(
        (id: UniqueIdentifier) => {
            if (id in items) {
                return id
            }

            return Object.keys(items).find(key => items[key].includes(id))
        },
        [items],
    )

    const getIndex = useCallback(
        (id: UniqueIdentifier) => {
            const container = findContainer(id)
            if (!container) return -1
            return items[container].indexOf(id)
        },
        [findContainer, items],
    )

    // Custom collision detection strategy
    const collisionDetectionStrategy: CollisionDetection = useCallback(
        args => {
            if (activeId && activeId in items) {
                return closestCenter({
                    ...args,
                    droppableContainers: args.droppableContainers.filter(
                        container => container.id in items,
                    ),
                })
            }

            // Find intersecting droppables
            const pointerIntersections = pointerWithin(args)
            const intersections =
                pointerIntersections.length > 0 ? pointerIntersections : rectIntersection(args)
            let overId = getFirstCollision(intersections, 'id')

            if (overId != null) {
                if (overId in items) {
                    const containerItems = items[overId]

                    if (containerItems.length > 0) {
                        // Return closest droppable within container
                        overId = closestCenter({
                            ...args,
                            droppableContainers: args.droppableContainers.filter(
                                container =>
                                    container.id !== overId &&
                                    containerItems.includes(container.id as any),
                            ),
                        })[0]?.id
                    }
                }

                lastOverId.current = overId
                return [{id: overId}]
            }

            // Handle case when draggable moves to a new container
            if (recentlyMovedToNewContainer.current) {
                lastOverId.current = activeId
            }

            // Return last match if no droppable is found
            return lastOverId.current ? [{id: lastOverId.current}] : []
        },
        [activeId, items],
    )

    // Drop animation
    const dropAnimation: DropAnimation = {
        sideEffects: defaultDropAnimationSideEffects({
            styles: {
                active: {
                    opacity: '0.5',
                },
            },
        }),
    }

    // Event handlers
    const onDragStart = ({active}: {active: any}) => {
        setActiveId(active.id)
        setClonedItems(items)
    }

    const onDragOver = ({active, over}: {active: any; over: any}) => {
        const overId = over?.id

        if (!overId || active.id in items) {
            return
        }

        const overContainer = findContainer(overId)
        const activeContainer = findContainer(active.id)

        if (!overContainer || !activeContainer) {
            return
        }

        if (activeContainer !== overContainer) {
            setItems(items => {
                const activeItems = items[activeContainer]
                const overItems = items[overContainer]
                const overIndex = overItems.indexOf(overId)
                const activeIndex = activeItems.indexOf(active.id)

                let newIndex: number

                if (overId in items) {
                    newIndex = overItems.length + 1
                } else {
                    const isBelowOverItem =
                        over &&
                        active.rect.current.translated &&
                        active.rect.current.translated.top > over.rect.top + over.rect.height

                    const modifier = isBelowOverItem ? 1 : 0

                    newIndex = overIndex >= 0 ? overIndex + modifier : overItems.length + 1
                }

                recentlyMovedToNewContainer.current = true

                return {
                    ...items,
                    [activeContainer]: items[activeContainer].filter(
                        item => item !== active.id,
                    ),
                    [overContainer]: [
                        ...items[overContainer].slice(0, newIndex),
                        items[activeContainer][activeIndex],
                        ...items[overContainer].slice(newIndex, items[overContainer].length),
                    ],
                }
            })
        }
    }

    const onDragEnd = ({active, over}: {active: any; over: any}) => {
        if (active.id in items && over?.id) {
            setContainers(containers => {
                const activeIndex = containers.indexOf(active.id)
                const overIndex = containers.indexOf(over.id)

                return arrayMove(containers, activeIndex, overIndex)
            })
        }

        const activeContainer = findContainer(active.id) as string

        if (!activeContainer) {
            setActiveId(null)
            return
        }

        const overId = over?.id

        if (!overId) {
            setActiveId(null)
            return
        }

        // Handle dropping on placeholder
        if (overId === PLACEHOLDER_ID) {
            const newContainerId = getNextContainerId()

            setContainers(containers => [...containers, newContainerId])
            setItems(items => ({
                ...items,
                [activeContainer]: items[activeContainer].filter(id => id !== activeId),
                [newContainerId]: [active.id],
            }))
            setActiveId(null)
            return
        }

        // Handle normal sorting
        const overContainer = findContainer(overId) as string

        if (overContainer) {
            const activeIndex = items[activeContainer].indexOf(active.id)
            const overIndex = items[overContainer].indexOf(overId)

            if (activeIndex !== overIndex || activeContainer !== overContainer) {
                setItems(items => {
                    const result = {
                        ...items,
                        [overContainer]: arrayMove(
                            items[overContainer].includes(active.id)
                                ? items[overContainer]
                                : [...items[overContainer], active.id],
                            items[overContainer].indexOf(active.id) === -1
                                ? items[overContainer].length
                                : items[overContainer].indexOf(active.id),
                            overIndex,
                        ),
                    }

                    // Remove from previous container if item moved between containers
                    if (activeContainer !== overContainer) {
                        result[activeContainer] = items[activeContainer as string].filter(
                            id => id !== active.id,
                        )
                    }

                    return result
                })
            }
        }

        setActiveId(null)
    }

    const onDragCancel = () => {
        if (clonedItems) {
            setItems(clonedItems)
        }
        setActiveId(null)
        setClonedItems(null)
    }

    // Utility functions
    const handleRemoveColumn = (containerId: UniqueIdentifier) => {
        setContainers(containers => containers.filter(id => id !== containerId))
        setItems(items => {
            const newItems = {...items}
            delete newItems[containerId as string]
            return newItems
        })
    }

    const getNextContainerId = useCallback(() => {
        const containerIds = Object.keys(items)
        const lastContainerId = containerIds[containerIds.length - 1]
        return String.fromCharCode(lastContainerId.charCodeAt(0) + 1)
    }, [items])

    const handleAddColumn = useCallback(() => {
        const newContainerId = getNextContainerId()
        setContainers(containers => [...containers, newContainerId])
        setItems(items => ({
            ...items,
            [newContainerId]: [],
        }))
    }, [getNextContainerId])

    // Render functions
    const renderSortableItemDragOverlay = useCallback(
        (id: UniqueIdentifier) => {
            // If custom render function is provided, use that
            if (renderItem) {
                return <div className={css.dragOverlayWrapper}>{renderItem(id)}</div>
            }

            // Otherwise use the default implementation
            return (
                <BoardItem
                    value={id}
                    handle={handle}
                    style={getItemStyles({
                        containerId: findContainer(id) as UniqueIdentifier,
                        overIndex: -1,
                        index: getIndex(id),
                        value: id,
                        isSorting: true,
                        isDragging: true,
                        isDragOverlay: true,
                    })}
                    color={getItemColor(id)}
                    wrapperStyle={wrapperStyle({index: 0})}
                    dragOverlay
                />
            )
        },
        [findContainer, getIndex, getItemStyles, handle, renderItem, wrapperStyle],
    )

    const renderContainerDragOverlay = useCallback(
        (containerId: UniqueIdentifier) => {
            return (
                <Column
                    id={containerId}
                    items={items[containerId as string]}
                    label={renderContainerHeader ? undefined : `Column ${containerId}`}
                    style={{
                        height: '100%',
                    }}
                    shadow
                    unstyled={false}
                >
                    {renderContainerHeader && (
                        <div className={css.containerHeader}>
                            {renderContainerHeader(containerId)}
                        </div>
                    )}
                    {items[containerId as string].map((item, index) => {
                        if (renderItem) {
                            return (
                                <div key={item} className={css.customItemWrapper}>
                                    {renderItem(item)}
                                </div>
                            )
                        }

                        return (
                            <BoardItem
                                key={item}
                                value={item}
                                handle={handle}
                                style={getItemStyles({
                                    containerId,
                                    overIndex: -1,
                                    index: getIndex(item),
                                    value: item,
                                    isDragging: false,
                                    isSorting: false,
                                    isDragOverlay: false,
                                })}
                                color={getItemColor(item)}
                                wrapperStyle={wrapperStyle({index})}
                            />
                        )
                    })}
                </Column>
            )
        },
        [
            getIndex,
            getItemStyles,
            handle,
            items,
            renderContainerHeader,
            renderItem,
            wrapperStyle,
        ],
    )

    // Reset recently moved flag
    useEffect(() => {
        requestAnimationFrame(() => {
            recentlyMovedToNewContainer.current = false
        })
    }, [items])

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={collisionDetectionStrategy}
            measuring={{
                droppable: {
                    strategy: MeasuringStrategy.Always,
                },
            }}
            onDragStart={onDragStart}
            onDragOver={onDragOver}
            onDragEnd={onDragEnd}
            onDragCancel={onDragCancel}
            cancelDrop={cancelDrop}
            modifiers={modifiers}
        >
            <div
                className={css.container}
                style={{
                    gridAutoFlow: vertical ? 'row' : 'column',
                }}
            >
                <SortableContext
                    items={[...containers, PLACEHOLDER_ID]}
                    strategy={
                        vertical ? verticalListSortingStrategy : horizontalListSortingStrategy
                    }
                >
                    {containers.map(containerId => (
                        <Column
                            key={containerId}
                            id={containerId}
                            label={renderContainerHeader ? undefined : `Column ${containerId}`}
                            items={items[containerId as string]}
                            scrollable
                            style={containerStyle}
                            onRemove={() => handleRemoveColumn(containerId)}
                        >
                            {renderContainerHeader && (
                                <div className={css.containerHeader}>
                                    {renderContainerHeader(containerId)}
                                </div>
                            )}
                            <SortableContext
                                items={items[containerId as string]}
                                strategy={strategy}
                            >
                                {items[containerId as string].map((value, index) => {
                                    if (renderItem) {
                                        return (
                                            <SortableItem
                                                key={value}
                                                id={value}
                                                index={index}
                                                handle={handle}
                                                disabled={isSortingContainer}
                                                wrapperStyle={wrapperStyle}
                                                containerId={containerId}
                                                getIndex={getIndex}
                                                customContent={renderItem(value)}
                                            />
                                        )
                                    }

                                    return (
                                        <SortableItem
                                            key={value}
                                            id={value}
                                            index={index}
                                            handle={handle}
                                            disabled={isSortingContainer}
                                            style={getItemStyles}
                                            wrapperStyle={wrapperStyle}
                                            containerId={containerId}
                                            getIndex={getIndex}
                                        />
                                    )
                                })}
                            </SortableContext>
                        </Column>
                    ))}
                </SortableContext>
            </div>

            {createPortal(
                <DragOverlay adjustScale={adjustScale} dropAnimation={dropAnimation}>
                    {activeId
                        ? containers.includes(activeId)
                            ? renderContainerDragOverlay(activeId)
                            : renderSortableItemDragOverlay(activeId)
                        : null}
                </DragOverlay>,
                document.body,
            )}
        </DndContext>
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
