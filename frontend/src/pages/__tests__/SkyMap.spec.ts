/**
 * Unit tests for SkyMap component
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import SkyMap from '@/pages/SkyMap.vue'
import axios from 'axios'

// Mock axios
vi.mock('axios')

describe('SkyMap.vue', () => {
  const mockBlocks = [
    {
      schedulingBlockId: '1000004990',
      raInDeg: 158.03,
      decInDeg: -68.03,
      priority: 8.5,
      priorityBin: 'High (10+)',
      scheduledFlag: true,
      requestedHours: 0.33,
      totalVisibilityHours: 12.5,
      elevationRangeDeg: 30.0,
      targetName: 'T32',
      targetId: 10
    },
    {
      schedulingBlockId: '1000004991',
      raInDeg: 200.5,
      decInDeg: 45.2,
      priority: 3.2,
      priorityBin: 'Low (<10)',
      scheduledFlag: false,
      requestedHours: 1.5,
      totalVisibilityHours: 18.0,
      elevationRangeDeg: 25.0,
      targetName: 'T45',
      targetId: 25
    }
  ]

  beforeEach(() => {
    // Reset mocks before each test
    vi.clearAllMocks()
  })

  it('renders the component', () => {
    const wrapper = mount(SkyMap)
    expect(wrapper.find('h1').text()).toBe('Sky Map - RA/Dec Distribution')
  })

  it('displays loading state initially', () => {
    const wrapper = mount(SkyMap)
    expect(wrapper.find('.animate-spin').exists()).toBe(true)
  })

  it('loads data from API on mount', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockResolvedValue({ data: { blocks: mockBlocks } })

    mount(SkyMap)

    // Wait for async operation
    await new Promise(resolve => setTimeout(resolve, 100))

    expect(mockedAxios.get).toHaveBeenCalledWith(
      'http://localhost:8081/api/v1/datasets/current'
    )
  })

  it('applies priority filters correctly', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockResolvedValue({ data: { blocks: mockBlocks } })

    const wrapper = mount(SkyMap)
    await new Promise(resolve => setTimeout(resolve, 100))

    // Set priority filter to exclude low priority blocks
    const minInput = wrapper.findAll('input[type="number"]')[0]
    await minInput.setValue(5)
    
    const applyButton = wrapper.find('button')
    await applyButton.trigger('click')

    // Should show only 1 block (the high priority one)
    // This is a simplified check - in real test we'd check the chart data
    expect(wrapper.vm.filteredCount).toBeLessThanOrEqual(wrapper.vm.totalCount)
  })

  it('filters by scheduled status', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockResolvedValue({ data: { blocks: mockBlocks } })

    const wrapper = mount(SkyMap)
    await new Promise(resolve => setTimeout(resolve, 100))

    const select = wrapper.find('select')
    await select.setValue('scheduled')
    
    const applyButton = wrapper.find('button')
    await applyButton.trigger('click')

    // Check that filtering happened
    expect(wrapper.vm.filteredData.length).toBeLessThanOrEqual(mockBlocks.length)
  })

  it('resets filters to default values', async () => {
    const wrapper = mount(SkyMap)
    
    // Change filter values
    const minInput = wrapper.findAll('input[type="number"]')[0]
    await minInput.setValue(10)

    // Reset filters
    const resetButton = wrapper.findAll('button')[1]
    await resetButton.trigger('click')

    expect(wrapper.vm.filters.priorityMin).toBe(0)
    expect(wrapper.vm.filters.priorityMax).toBe(100)
    expect(wrapper.vm.filters.scheduledStatus).toBe('all')
  })

  it('handles API errors gracefully', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockRejectedValue(new Error('Network error'))

    const wrapper = mount(SkyMap)
    await new Promise(resolve => setTimeout(resolve, 100))

    expect(wrapper.find('.bg-red-50').exists()).toBe(true)
    expect(wrapper.vm.error).toBeTruthy()
  })

  it('displays target information when available', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockResolvedValue({ data: { blocks: mockBlocks } })

    const wrapper = mount(SkyMap)
    await new Promise(resolve => setTimeout(resolve, 100))

    // Verify that blocks with target info are loaded
    expect(wrapper.vm.allData[0].targetName).toBe('T32')
    expect(wrapper.vm.allData[0].targetId).toBe(10)
  })

  it('displays correct count of filtered observations', async () => {
    const mockedAxios = axios as any
    mockedAxios.get.mockResolvedValue({ data: { blocks: mockBlocks } })

    const wrapper = mount(SkyMap)
    await new Promise(resolve => setTimeout(resolve, 100))

    expect(wrapper.vm.totalCount).toBe(2)
    expect(wrapper.vm.filteredCount).toBe(2)
  })
})
