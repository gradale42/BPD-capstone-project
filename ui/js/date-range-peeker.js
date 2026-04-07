// Date Range Picker with time support (Grafana-like)
class DateRangePeeker {
    constructor(elementId, onChangeCallback) {
        this.elementId = elementId;
        this.onChange = onChangeCallback;
        this.init();
    }

    init() {
        const start = moment().subtract(29, 'days');
        const end = moment();

        const cb = (start, end) => {
            $(`#${this.elementId} span`).html(start.format('DD MMM YYYY HH:mm') + ' — ' + end.format('DD MMM YYYY HH:mm'));
            if (this.onChange) this.onChange(start, end);
        };

        $(`#${this.elementId}`).daterangepicker({
            startDate: start,
            endDate: end,
            timePicker: true,
            timePickerIncrement: 15,
            timePicker24Hour: true,
            minDate: moment('2010-01-01'),
            maxDate: moment(),
            ranges: {
                'Last 24h': [moment().subtract(24, 'hours'), moment()],
                'Last 7d': [moment().subtract(6, 'days'), moment()],
                'Last 30d': [moment().subtract(29, 'days'), moment()],
                'This month': [moment().startOf('month'), moment().endOf('month')],
                'Last month': [moment().subtract(1, 'month').startOf('month'), moment().subtract(1, 'month').endOf('month')],
                'All 2024': [moment('2024-01-01'), moment('2024-12-31')],
                'All 2025': [moment('2025-01-01'), moment('2025-12-31')]
            },
            locale: {
                format: 'DD/MM/YYYY HH:mm',
                separator: ' — ',
                applyLabel: 'Apply',
                cancelLabel: 'Cancel',
                fromLabel: 'From',
                toLabel: 'To',
                customRangeLabel: 'Custom',
                daysOfWeek: ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
                monthNames: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
            }
        }, cb);

        cb(start, end);
    }

    getCurrentRange() {
        const picker = $(`#${this.elementId}`).data('daterangepicker');
        return { startDate: picker.startDate, endDate: picker.endDate };
    }
}