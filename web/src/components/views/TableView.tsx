// web/src/components/views/TableView.tsx
import {useCallback, useState} from 'react'
import {useKanbanData} from 'src/components/context/KanbanDataProvider'
import {CardForm} from 'src/components/card/CardForm'
import css from './TableView.module.css'

export function TableView() {
    const {boardData, moveCard, addCard} = useKanbanData()
    const [isAddingCard, setIsAddingCard] = useState(false)
    const [newCardColumn, setNewCardColumn] = useState<string>('')
    const [sortField, setSortField] = useState<string | null>(null)
    const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

    // Don't render anything until we have data
    if (!boardData) {
        return (
            <div className={css.loading}>
                <div className={css.pulse}></div>
            </div>
        )
    }

    // Extract all cards
    const allCards = Object.values(boardData.cards)

    // Format columns for display
    const columns = boardData.columns || []

    // Set default column if not set yet
    if (columns.length > 0 && !newCardColumn) {
        setNewCardColumn(columns[0].id)
    }

    // Handle sorting
    const handleSort = (field?: string) => {
        if (!field) {
            return
        }
        if (sortField === field) {
            // Toggle direction
            setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc')
        } else {
            // New sort field
            setSortField(field)
            setSortDirection('asc')
        }
    }

    // Apply sorting to cards
    const getSortedCards = () => {
        if (!sortField) return allCards

        return [...allCards].sort((a, b) => {
            let valueA: string
            let valueB: string

            if (sortField === 'title') {
                valueA = a.title
                valueB = b.title
            } else if (sortField === 'description') {
                valueA = a.description
                valueB = b.description
            } else {
                // Property field
                valueA = a.properties[sortField] || ''
                valueB = b.properties[sortField] || ''
            }

            const comparison = valueA.localeCompare(valueB)
            return sortDirection === 'asc' ? comparison : -comparison
        })
    }

    // Handle moving cards between columns
    const handleMoveToColumn = (cardId: string, columnId: string) => {
        if (!boardData.groupByProperty) return

        // Find the card to get its current column
        const card = boardData.cards[cardId]
        if (card) {
            const currentColumn = card.properties[boardData.groupByProperty]

            if (currentColumn !== columnId) {
                moveCard(cardId, currentColumn, columnId)
            }
        }
    }

    // Handle adding a new card
    const handleAddCard = (title: string, description: string, columnId: string) => {
        if (title.trim() && columnId) {
            const success = addCard(title.trim(), description.trim(), columnId)

            if (success) {
                // Reset form
                setIsAddingCard(false)
            }
            return success
        }
        return false
    }

    // Render sort icon based on current sort state
    const renderSortIcon = (field: string) => {
        if (sortField !== field) {
            return (
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                >
                    <path d="m7 15 5 5 5-5"></path>
                    <path d="m7 9 5-5 5 5"></path>
                </svg>
            )
        }

        return sortDirection === 'asc' ? (
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
            >
                <path d="m18 15-6-6-6 6"></path>
            </svg>
        ) : (
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
            >
                <path d="m6 9 6 6 6-6"></path>
            </svg>
        )
    }

    // Build table headers dynamically based on properties
    const generateTableHeaders = () => {
        // Always include checkmark, title, and status columns
        const headers = [
            <th key="checkbox" className={css.checkboxColumn}>
                <div className={css.cellWrapper}>
                    <div className={css.tableHeader}>Done</div>
                </div>
            </th>,
            <th key="title" onClick={() => handleSort('title')}>
                <div className={css.cellWrapper}>
                    <div className={css.tableHeader}>
                        <div>Title</div>
                        <div className={css.headerSort}>{renderSortIcon('title')}</div>
                    </div>
                </div>
            </th>,
        ]

        // Always include the group by property (Status, State, etc.)
        if (boardData.groupByProperty) {
            const {groupByProperty} = boardData
            headers.push(
                <th
                    key={boardData.groupByProperty}
                    onClick={() => handleSort(groupByProperty)}
                >
                    <div className={css.cellWrapper}>
                        <div className={css.tableHeader}>
                            <div>{boardData.groupByProperty}</div>
                            <div className={css.headerSort}>
                                {renderSortIcon(boardData.groupByProperty)}
                            </div>
                        </div>
                    </div>
                </th>,
            )
        }

        // Add other properties as columns (except the grouping property which is already added)
        boardData.properties.forEach(prop => {
            if (prop.name !== boardData.groupByProperty) {
                headers.push(
                    <th key={prop.name} onClick={() => handleSort(prop.name)}>
                        <div className={css.cellWrapper}>
                            <div className={css.tableHeader}>
                                <div>{prop.name}</div>
                                <div className={css.headerSort}>
                                    {renderSortIcon(prop.name)}
                                </div>
                            </div>
                        </div>
                    </th>,
                )
            }
        })

        // Always include description at the end
        headers.push(
            <th key="description" onClick={() => handleSort('description')}>
                <div className={css.cellWrapper}>
                    <div className={css.tableHeader}>
                        <div>Description</div>
                        <div className={css.headerSort}>{renderSortIcon('description')}</div>
                    </div>
                </div>
            </th>,
        )

        return headers
    }

    // Generate table rows dynamically
    const generateTableRows = () => {
        const sortedCards = getSortedCards()
        const groupProp = boardData.groupByProperty

        if (sortedCards.length === 0) {
            return (
                <tr>
                    <td
                        colSpan={boardData.properties.length + 3}
                        style={{textAlign: 'center', padding: '20px'}}
                    >
                        No cards found
                    </td>
                </tr>
            )
        }

        return sortedCards.map(card => {
            // Determine if this card is marked as "Done"
            const isDone = groupProp && card.properties[groupProp] === 'Done'

            return (
                <tr key={card.id} className={isDone ? css.isComplete : ''}>
                    <td>
                        <div className={css.cellWrapper}>
                            <input
                                type="checkbox"
                                checked={Boolean(isDone)}
                                onChange={() => {
                                    if (groupProp) {
                                        const newStatus = isDone ? 'In Progress' : 'Done'
                                        handleMoveToColumn(card.id, newStatus)
                                    }
                                }}
                            />
                        </div>
                    </td>
                    {groupProp && (
                        <td>
                            <div className={css.cellWrapper}>
                                <div className={css.statusCell}>
                                    <div>{card.properties[groupProp]}</div>
                                    <div className={css.statusSelector}>
                                        <select
                                            value={card.properties[groupProp]}
                                            onChange={e =>
                                                handleMoveToColumn(card.id, e.target.value)
                                            }
                                            className={css.statusSelect}
                                        >
                                            {columns.map(column => (
                                                <option key={column.id} value={column.id}>
                                                    {column.title}
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </td>
                    )}
                    {boardData.properties
                        .filter(prop => prop.name !== groupProp)
                        .map(prop => (
                            <td key={prop.name}>
                                <div className={css.cellWrapper}>
                                    {card.properties[prop.name] || ''}
                                </div>
                            </td>
                        ))}
                    <td>
                        <div className={css.cellWrapper}>{card.description}</div>
                    </td>
                </tr>
            )
        })
    }

    return (
        <div className={css.tableView}>
            <div className={css.controls}>
                <button className={css.addButton} onClick={() => setIsAddingCard(true)}>
                    <span className={css.buttonIcon}>+</span> Add Card
                </button>
            </div>

            {isAddingCard && (
                <div className={css.addForm}>
                    <div className={css.formHeader}>
                        <h3>Add New Card</h3>
                        <button
                            className={css.closeButton}
                            onClick={() => setIsAddingCard(false)}
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
                        columnOptions={columns}
                        columnId={newCardColumn}
                        onAddCard={handleAddCard}
                        onCancel={() => setIsAddingCard(false)}
                        onColumnChange={setNewCardColumn}
                    />
                </div>
            )}

            <div className={css.wrapper}>
                <table>
                    <thead>
                        <tr>{generateTableHeaders()}</tr>
                    </thead>
                    <tbody>{generateTableRows()}</tbody>
                </table>
            </div>
        </div>
    )
}
