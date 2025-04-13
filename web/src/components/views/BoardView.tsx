// web/src/components/views/BoardView.tsx
import {useState, useEffect} from 'react'
import {UniqueIdentifier} from '@dnd-kit/core'
import {CardForm} from 'src/components/card/CardForm'
import {useKanbanData} from 'src/components/context/KanbanDataProvider'
import css from './BoardView.module.css'
import {KanbanBoard} from 'src/components/board/KanbanBoard'

export function KanbanBoardView() {
    const {boardData, moveCard, addCard} = useKanbanData()
    const [activeAddColumn, setActiveAddColumn] = useState<string | null>(null)

    useEffect(() => {
        console.debug('[KanbanBoardView] Board data changed:', boardData)
        if (boardData && boardData.items) {
            console.debug('[KanbanBoardView] Items:', boardData.items)
            console.debug('[KanbanBoardView] Cards:', boardData.cards)
            console.debug('[KanbanBoardView] Columns:', boardData.columns)
        }
    }, [boardData])

    // Don't render anything until we have data
    if (!boardData || !boardData.items) {
        console.debug('[KanbanBoardView] Loading - no board data yet')
        return (
            <div className={css.loading}>
                <div className={css.pulse}></div>
            </div>
        )
    }

    // Additional check to ensure data is in expected format
    if (Object.keys(boardData.items).length === 0) {
        console.debug('[KanbanBoardView] No items in board data')
        return (
            <div className={css.loading}>
                <p>No items found in board data</p>
            </div>
        )
    }

    // Handle DnD-specific item changes
    const handleItemsChange = (newItems: Record<string, UniqueIdentifier[]>) => {
        if (!boardData) return
        console.debug('[KanbanBoardView] Items changed:', newItems)

        // Compare with previous items to find moved cards
        Object.entries(newItems).forEach(([containerId, cardIds]) => {
            cardIds.forEach(cardId => {
                const cardIdStr = String(cardId)
                // Find which container this card was previously in
                const previousContainerId = Object.entries(boardData.items || {}).find(
                    ([colId, ids]) => ids.includes(cardIdStr) && colId !== containerId,
                )?.[0]

                if (previousContainerId) {
                    // Card moved to a new column
                    console.debug(
                        `[KanbanBoardView] Moving card ${cardIdStr} from ${previousContainerId} to ${containerId}`,
                    )
                    moveCard(cardIdStr, previousContainerId, containerId)
                }
            })
        })
    }

    // Handle adding a new card
    const handleAddNewCard = (columnId: string, title: string, description: string) => {
        console.debug(`[KanbanBoardView] Adding new card to ${columnId}: ${title}`)
        const result = addCard(title, description, columnId)
        if (result) {
            setActiveAddColumn(null)
        }
        return result
    }

    // Format columns for UI display
    const columns = boardData.columns.map(column => ({
        id: column.id,
        title: column.title,
    }))

    // Custom item renderer for cards
    const renderItem = (id: UniqueIdentifier) => {
        if (!boardData) return <div>Loading...</div>

        const cardId = String(id)
        console.debug(`[KanbanBoardView] Rendering card ${cardId}`)

        const card = boardData.cards[cardId]
        if (!card) {
            console.error(`[KanbanBoardView] Card not found:`, cardId, boardData.cards)
            return <div>Card {cardId} not found!</div>
        }

        return (
            <div className={css.cardContent}>
                <div className={css.cardTitle}>{card.title}</div>
                {card.description && (
                    <div className={css.cardDescription}>{card.description}</div>
                )}
                <div className={css.cardProperties}>
                    {Object.entries(card.properties || {}).map(
                        ([key, value]) =>
                            key !== boardData.groupByProperty && (
                                <div key={key} className={css.cardProperty}>
                                    <span className={css.propertyKey}>{key}:</span>
                                    <span className={css.propertyValue}>{String(value)}</span>
                                </div>
                            ),
                    )}
                </div>
            </div>
        )
    }

    // Custom header renderer for columns
    const renderContainerHeader = (containerId: UniqueIdentifier) => {
        const containerIdStr = String(containerId)
        console.debug(`[KanbanBoardView] Rendering container header for ${containerIdStr}`)

        const column = columns.find(col => col.id === containerIdStr)
        if (!column) {
            console.error(`[KanbanBoardView] Column not found:`, containerIdStr, columns)
            return <div>Column {containerIdStr} not found!</div>
        }

        const cardCount = boardData?.items[containerIdStr]?.length || 0

        return (
            <div className={css.columnHeader}>
                <div className={css.columnTitle}>{column.title}</div>
                <div className={css.columnCount}>{cardCount}</div>
                <button
                    className={css.addCardButton}
                    onClick={() => setActiveAddColumn(containerIdStr)}
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <line x1="12" y1="5" x2="12" y2="19"></line>
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                    </svg>
                </button>
            </div>
        )
    }

    console.debug('[KanbanBoardView] Rendering board with items:', boardData.items)

    return (
        <div className={css.boardView}>
            {/* Add Card Modal */}
            {activeAddColumn && (
                <div className={css.addCardModal}>
                    <div className={css.addCardModalContent}>
                        <div className={css.addCardModalHeader}>
                            <h3>
                                Add Card to{' '}
                                {columns.find(col => col.id === activeAddColumn)?.title}
                            </h3>
                            <button
                                className={css.closeModalButton}
                                onClick={() => setActiveAddColumn(null)}
                            >
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="20"
                                    height="20"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    strokeWidth="2"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                >
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>
                        </div>
                        <CardForm
                            columnId={activeAddColumn}
                            onAddCard={(title, description, columnId) =>
                                handleAddNewCard(columnId, title, description)
                            }
                            onCancel={() => setActiveAddColumn(null)}
                        />
                    </div>
                </div>
            )}

            {/* Kanban Board */}
            <KanbanBoard
                initialItems={boardData.items}
                renderItem={renderItem}
                renderContainerHeader={renderContainerHeader}
                onItemsChange={handleItemsChange}
                getItemStyles={() => ({})}
                adjustScale={true}
                handle={false}
                containerStyle={{
                    minWidth: '280px',
                    maxWidth: '350px',
                    marginRight: '10px',
                }}
                wrapperStyle={() => ({})}
            />
        </div>
    )
}
