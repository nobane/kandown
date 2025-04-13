// web/src/components/board/stories/KanbanBoard.stories.tsx
// @file web/src/components/stories/KanbanBoard.stories.tsx
import React from 'react';
import { Meta, StoryObj } from '@storybook/react';
import { UniqueIdentifier } from '@dnd-kit/core';
import { createRange } from 'src/utils/helpers';
import {KanbanBoard} from 'src/components/board/KanbanBoard';

const meta: Meta<typeof KanbanBoard> = {
  title: 'Components/KanbanBoard',
  component: KanbanBoard,
  parameters: {
    layout: 'centered',
  },
  argTypes: {
    initialItems: { control: 'object' },
    adjustScale: { control: 'boolean' },
    handle: { control: 'boolean' },
    containerStyle: { control: 'object' },
  },
};

export default meta;
type Story = StoryObj<typeof KanbanBoard>;

const baseItems = {
    Backlog: createRange(3, index => `card_${index}`),
    'In Progress': createRange(2, index => `card_${index + 3}`),
    Done: createRange(2, index => `card_${index + 5}`),
}

// Custom renderer for cards - now properly typed with UniqueIdentifier
const renderItem = (id: UniqueIdentifier) => {
    // Extract card number
    const idStr = String(id);
    const match = idStr.match(/card_(\d+)/)
    const cardNum = match ? parseInt(match[1]) : 0

    return (
        <div style={{padding: '12px', width: '100%'}}>
            <div style={{fontWeight: 'bold', marginBottom: '4px'}}>Card {cardNum}</div>
            <div style={{fontSize: '0.8rem', color: '#666'}}>
                This is a sample card description
            </div>
            <div
                style={{
                    marginTop: '8px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '2px',
                    fontSize: '0.8rem',
                }}
            >
                <div>
                    <span style={{fontWeight: 'bold', marginRight: '4px'}}>Owner:</span>
                    <span>User {(cardNum % 3) + 1}</span>
                </div>
                <div>
                    <span style={{fontWeight: 'bold', marginRight: '4px'}}>Priority:</span>
                    <span>{['Low', 'Medium', 'High'][cardNum % 3]}</span>
                </div>
            </div>
        </div>
    )
}

// Custom renderer for column headers - properly typed with UniqueIdentifier
const renderContainerHeader = (containerId: UniqueIdentifier) => {
    const containerIdStr = String(containerId);

    return (
        <div
            style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '8px 12px',
                width: '100%',
            }}
        >
            <div style={{fontWeight: 'bold'}}>{containerIdStr}</div>
            <div
                style={{
                    background: '#f0f0f0',
                    borderRadius: '10px',
                    padding: '2px 8px',
                    fontSize: '0.8rem',
                }}
            >
                {baseItems[containerIdStr as keyof typeof baseItems]?.length || 0}
            </div>
        </div>
    )
}

// Basic example
export const Default: Story = {
    // web/src/components/stories/KanbanBoard.stories.tsx (continued)
    args: {
        initialItems: baseItems,
        renderItem,
        renderContainerHeader,
        getItemStyles: () => ({}),
        adjustScale: true,
        handle: false,
        containerStyle: {
            minWidth: '280px',
            maxWidth: '350px',
            marginRight: '10px',
        },
        wrapperStyle: () => ({}),
        onItemsChange: items => console.debug('Items changed:', items),
    },
}

// With drag handles
export const WithHandles: Story = {
    args: {
        ...Default.args,
        handle: true,
    },
}

// Vertical layout
export const VerticalLayout: Story = {
    args: {
        ...Default.args,
        vertical: true,
    },
}

// Scrollable columns
export const ScrollableColumns: Story = {
    args: {
        ...Default.args,
        initialItems: {
            Backlog: createRange(8, index => `card_${index}`),
            'In Progress': createRange(6, index => `card_${index + 8}`),
            Done: createRange(5, index => `card_${index + 14}`),
        },
    },
}

// Custom styling
export const CustomStyling: Story = {
    args: {
        ...Default.args,
        containerStyle: {
            minWidth: '300px',
            maxWidth: '350px',
            marginRight: '16px',
            backgroundColor: '#f9f9f9',
            borderRadius: '8px',
            border: '1px solid #e0e0e0',
            boxShadow: '0 2px 4px rgba(0,0,0,0.05)',
        },
        getItemStyles: ({isDragging}: {isDragging: boolean}) => ({
            backgroundColor: isDragging ? '#f0f8ff' : '#ffffff',
            boxShadow: isDragging
                ? '0 5px 10px rgba(0,0,0,0.15)'
                : '0 1px 3px rgba(0,0,0,0.1)',
            borderLeft: '3px solid #6366f1',
            borderRadius: '4px',
            padding: '12px',
        }),
    },
}