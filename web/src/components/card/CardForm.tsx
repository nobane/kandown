// web/src/components/card/CardForm.tsx
import {useState} from 'react'
import css from './CardForm.module.css'

interface Column {
    id: string
    title: string
}

interface CardFormProps {
    columnId: string
    columnOptions?: Column[]
    onAddCard: (title: string, description: string, columnId: string) => boolean
    onCancel: () => void
    onColumnChange?: (columnId: string) => void
}

export function CardForm({
    columnId,
    columnOptions,
    onAddCard,
    onCancel,
    onColumnChange,
}: CardFormProps) {
    const [title, setTitle] = useState('')
    const [description, setDescription] = useState('')
    const [isSubmitting, setIsSubmitting] = useState(false)
    const [selectedColumnId, setSelectedColumnId] = useState(columnId)

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()

        if (!title.trim()) return

        setIsSubmitting(true)
        const success = onAddCard(title.trim(), description.trim(), selectedColumnId)

        if (!success) {
            setIsSubmitting(false)
        }
    }

    const handleColumnChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const newColumnId = e.target.value
        setSelectedColumnId(newColumnId)
        if (onColumnChange) {
            onColumnChange(newColumnId)
        }
    }

    return (
        <form onSubmit={handleSubmit} className={css.form}>
            <input
                className={css.input}
                type="text"
                placeholder="Card title..."
                value={title}
                onChange={e => setTitle(e.target.value)}
                disabled={isSubmitting}
                autoFocus
            />

            <textarea
                className={css.textarea}
                placeholder="Description (optional)"
                value={description}
                onChange={e => setDescription(e.target.value)}
                disabled={isSubmitting}
                rows={3}
            />

            {columnOptions && (
                <select
                    className={css.select}
                    value={selectedColumnId}
                    onChange={handleColumnChange}
                    disabled={isSubmitting}
                >
                    {columnOptions.map(column => (
                        <option key={column.id} value={column.id}>
                            {column.title}
                        </option>
                    ))}
                </select>
            )}

            <div className={css.actions}>
                <button
                    type="submit"
                    className={css.addButton}
                    disabled={isSubmitting || !title.trim()}
                >
                    Add card
                </button>
                <button
                    type="button"
                    className={css.cancelButton}
                    onClick={onCancel}
                    disabled={isSubmitting}
                >
                    Cancel
                </button>
            </div>
        </form>
    )
}
