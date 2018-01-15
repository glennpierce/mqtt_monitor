// from collections import defaultdict
// import sys, logging, datetime
// import arrow, os, os.path
// from celery.task import task
// from django.conf import settings
// from itertools import groupby


// import timing

// if __name__ == "__main__":
//     thrift_path = os.path.join(os.path.dirname(__file__), '/opt/bcp/bcp_apps/bmos/')
//     sys.path.append(thrift_path)
//     thrift_path = os.path.join(os.path.dirname(__file__), '/opt/bcp/bcp_apps/bmos/.thrift/bmos_python/')
//     sys.path.append(thrift_path)
//     from ThriftBmosService.ttypes import DataExtractOptions, DataType
//     import BmosServerConnection
//     #DJANGO_SETTINGS_MODULE=/opt/bcp/bcp/bcp/settings.py
//     from_email = 'support@carnegosystems.com'
// else:
//     from ThriftBmosService.ttypes import DataExtractOptions, DataType
//     from bcp_apps.bmos.BmosServerConnection import BmosServerConnection
//     from_email = settings.REPORTS_DEFAULT_FROM_EMAIL

// from django.utils import timezone
// from django.utils.timezone import make_aware
// from django.contrib import messages
// from django.core.mail import EmailMessage, get_connection, EmailMultiAlternatives
// import models

// try:
//     from django.utils.encoding import smart_text # For Django >= 1.5
// except ImportError:
//     from django.utils.encoding import smart_unicode as smart_text


// SERVER_NAME = settings.SERVER_NAME

// def email_debug(message, connection=None):
//     subject = 'Report DEBUG Email ' +  arrow.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")
//     msg = EmailMessage(subject=subject, body=message, from_email=from_email,
//                        to=['glennpierce@gmail.com'], connection=connection)

//     msg.send(fail_silently=False)

// def email_message(details, attachment=None, attachment_filename=None, connection=None):
//     subject = SERVER_NAME + ' ' + smart_text(details['report']['subject']) + ' ' +  arrow.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")

//     logging.info("recipients: %s type: %s", str(details['report']['recipients']), type(details['report']['recipients']))

//     msg = EmailMessage(subject=subject, body=details['report']['message'], from_email=from_email,
//                        to=details['report']['recipients'], connection=connection,
//                        headers=details['report']['headers'])

//     if attachment:
//         msg.attach(attachment_filename, attachment, 'text/plain')

//     logging.info("Emailing Message %s %s", details['report']['recipients'], msg)

//     log_msg = msg.from_email + "\n\n" + msg.subject + "\n\n" + msg.body + "\n\n"

//     if attachment:
//         log_msg += attachment

//     recipients = ','.join(details['report']['recipients'])
//     log = models.ReportLog(name=details['report']['name'],
//                            send_time=make_aware(datetime.datetime.now(), timezone.get_default_timezone()),
//                            log=log_msg,
//                            recipients=recipients)
//     log.save()

//     return msg


// from django.test.utils import override_settings

// @override_settings(EMAIL_BACKEND='django.core.mail.backends.smtp.EmailBackend')
// def dispatch(details, attachment=None, attachment_filename=None, connection=None):
//         """
//         Actually send out the email and log the result
//         """
//         try:
//             msg = email_message(details, attachment=attachment, attachment_filename=attachment_filename)
//             logging.info(msg)
//             msg.send(fail_silently=False)

//         except Exception as err:
//             logging.warning("Emailing Error")
//             logging.warning(str(err))


// class AggregationType(object):
//     Sum = 1
//     Diff = 2


// def run_report_for_sensor(sensor_id, sensor_name, start, aggregation_type=AggregationType.Diff, dt=None):
//     bmos = BmosServerConnection(host=settings.BMOS_HOSTS[0])
//     auth_details = bmos.authenticate(settings.BMOS_USER, settings.BMOS_PASS)

//     if not dt:
//         now = arrow.utcnow()
//         today = now.floor('day')
//     else:
//         today = dt.floor('day')

//     options = DataExtractOptions(startTimestamp=start.format("X.SSSSSS"),
//                                  endTimestamp=today.format("X.SSSSSS"),
//                                  includeMilliSecondsInOutput=False,
//                                  dataType=DataType.DataOriginal,
//                                  compress=False,
//                                  nullReturn=False,
//                                  nullReturnString='0.0',
//     )

//     timing.start()
//     # Check if there is cleaned data
//     if bmos.hasSensorIdsGotCleanedData(bmos.auth_token, [sensor_id]):
//         logging.info("getting cleaned data for report")
//         options.dataType = DataType.DataCleaned
//     else:
//         logging.info("No cleaned data for sensor %s", sensor_id)    
   
//     timing.finish()

//     time_taken = float(timing.micro()) / 1000000
//     logging.debug("hasSensorIdsGotCleanedData for sensor_id %s %f seconds", sensor_id, time_taken)

//     timing.start()

//     data = bmos.getSensorData(bmos.auth_token, sensor_id, options)

//     if not data:
//         logging.warning("No data for sensor id %s options %s", sensor_id, options)
//         #email_debug("No data for sensor id %s options %s" % (sensor_id,options))

//     daily_values = {}

//     timing.finish()

//     time_taken = float(timing.micro()) / 1000000
//     logging.debug("getSensorData for sensor_id %s %f seconds", sensor_id, time_taken)


//     timing.start()

//     for r in data:
//         arrow_utc_time = arrow.Arrow.strptime(r.timestamp, "%Y-%m-%dT%H:%M:%S+0000")
//         day = arrow_utc_time.strftime("%d/%m/%Y")

//         #print "day", arrow_utc_time
//         utc_unix_timestamp = int(arrow_utc_time.format('X'))
//         utc_offset_from_start = utc_unix_timestamp - int(start.format('X'))
//         half_hour = int(utc_offset_from_start / 1800)  # Every half hour

//         if day not in daily_values:
//             daily_values[day] = defaultdict(list)

//         daily_values[day][half_hour].append((arrow_utc_time, r.values[0]))

//     text_data = ""

//     sorted_daily_items = daily_values.items()
//     sorted_daily_items.sort(key=lambda i: datetime.datetime.strptime(i[0], '%d/%m/%Y'))

//     for day, half_hours in sorted_daily_items:
//         if aggregation_type == AggregationType.Diff:
//             values = sorted(half_hours.values())
//             values = [(v[0], v[-1]) for v in values]
//             #logging.info(values)
//             values = [str(v[1][1]-v[0][1]) for v in values]
//         else:
//             values = sorted(half_hours.values())
//             values = [str(sum(v)) for v in values]

//         text_data += "%s,%s,%s" % (sensor_name, day, ','.join(values) + '\n')

//     timing.finish()

//     time_taken = float(timing.micro()) / 1000000
//     logging.info("formatting to csv for sensor_id %s %f seconds", sensor_id, time_taken)

//     return text_data


// def run_report_for_sensor2(sensor_id, sensor_name, start, aggregation_type=AggregationType.Diff, dt=None):
//     bmos = BmosServerConnection(host=settings.BMOS_HOSTS[0])
//     auth_details = bmos.authenticate(settings.BMOS_USER, settings.BMOS_PASS)

//     if not dt:
//         now = arrow.utcnow()
//         today = now.floor('day')
//     else:
//         today = dt.floor('day')

//     # To get the diffence from the last half hour we need to start start back a little
//     # so we have have a value to substract from our first.
//     start -= datetime.timedelta(minutes=30)

//     # start should be today (floored) minus 7 days so also floored (00:00)
//     options = DataExtractOptions(startTimestamp=start.format("X.SSSSSS"),
//                                  endTimestamp=today.format("X.SSSSSS"),
//                                  includeMilliSecondsInOutput=False,
//                                  dataType=DataType.DataOriginal,
//                                  compress=False,
//                                  nullReturn=False,
//                                  nullReturnString='0.0',
//     )

//     # Check if there is cleaned data
//     if bmos.hasSensorIdsGotCleanedData(bmos.auth_token, [sensor_id]):
//         logging.info("Getting cleaned data for report")
//         options.dataType = DataType.DataCleaned

//     # getSensorData comes back as ordered
//     data = bmos.getSensorData(bmos.auth_token, sensor_id, options)

//     if not data:
//         return ""

//     def offset_embed(item):
//         # Add useful instance members dynamically (they don't exist in the object at start time)
//         item.arrow_utc_time = arrow.Arrow.strptime(item.timestamp, "%Y-%m-%dT%H:%M:%S+0000")
//         item.day = item.arrow_utc_time.strftime("%d/%m/%Y")
//         utc_unix_timestamp = int(item.arrow_utc_time.format('X'))
//         start_of_day = int(item.arrow_utc_time.floor('day').format('X'))
//         utc_offset_from_start_of_day = utc_unix_timestamp - start_of_day
//         item.half_hour = int(utc_offset_from_start_of_day / 1800)  # Every half hour
//         item.value = item.values[0]        # We only deal with one sensor so only ever one value
//         return (item.day, item.half_hour)  # Return variable to group by. So we get a list of groups.
//                                            # Each group is the data for a halfhour interval on a particular day
 
//     halfhours = []
//     last_value = 0
//     for key, group in groupby(data, offset_embed):
//         group = list(group) # Makes reiterating possible
//         day, halfhour = key
//         summed_values = sum([i.value for i in group])
//         diff_from_last = group[-1].value - last_value
//         last_value = group[-1].value
//         halfhours.append((day, halfhour, summed_values, diff_from_last))


//     # Ok we now have a list of all halfhours. Each half hour has the summbed values for that half hour and
//     # the difference from the previous half hour value
//     # First entry will not be valid as we extracted halfhour early so we could calculate the difference
//     # between half hours
//     halfhours.pop(0)  # Remove fist element (halfhour)

//     text_data = ""
//     value_index = 2
//     if aggregation_type == AggregationType.Diff:
//         value_index = 3

//     for day, group in groupby(halfhours, lambda x: x[0]):
//         # Need to pad if there are missing half hours
//         values = ['NaN' for i in range(0,48)]   
//         for i in group:
//             values[i[1]] = str(i[value_index])
//         text_data += "%s,%s,%s" % (sensor_name, day, ','.join(values) + '\n')

//     return text_data


// #@task
// def run_report(details, dt=None):
//     logging.info("run_report task")

//     bmos = BmosServerConnection(host=settings.BMOS_HOSTS[0])
//     auth_details = bmos.authenticate(settings.BMOS_USER, settings.BMOS_PASS)

//     report_name = details['report']['name']
//     sensors = details['report']['sensor_ids']

//     logging.info("running report send for %s", report_name)

//     # prints header like meter #, date, units, 00:00, 00:30, 01:00, 01:30, 02:00 22:30, 23:00, 23:30
//     text_data = 'meter,date,'

//     if dt:
//         today = dt.floor('day')
//     else:
//         now = arrow.utcnow()
//         today = now.floor('day')
    
//     start = today - datetime.timedelta(days=7)

//     # Add times for header
//     tmp = today
//     while tmp < today.ceil('day'):
//         text_data += (tmp.strftime('%H:%M') + ',')
//         tmp += datetime.timedelta(seconds=1800)

//     text_data = text_data[:-1] + '\n'   # Removes last ,

//     # Get data for all sensors
//     for sensor_id, sensor_name in sensors:
//         logging.info("run_report sensor %s", sensor_name)
//         text_data += run_report_for_sensor2(sensor_id, sensor_name, start, dt=today)

//     filename = report_name + '-' + today.format('YYYY-MM-DD') + '.csv'

//     logging.info("report details: %s", str(details))
//     dispatch(details, attachment=text_data, attachment_filename=filename)



// def resend_log(report_log):
//     logging.info("run_report task")

//     bmos = BmosServerConnection(host=settings.BMOS_HOSTS[0])
//     auth_details = bmos.authenticate(settings.BMOS_USER, settings.BMOS_PASS)

//     from_email = settings.REPORTS_DEFAULT_FROM_EMAIL
//     recipients = [i.strip() for i in report_log.recipients.split(',')]
//     report_name = report_log.name
//     logging.info("running report send for %s", report_name)

//     now = arrow.utcnow()
//     today = now.floor('day')

//     attachment_filename = report_name + '-' + today.format('YYYY-MM-DD') + '.csv'
//     attachment = report_log.log

//     subject = "Resending report for %s %s" % (report_name, report_log.send_time)

//     msg = EmailMessage(subject=subject, body=subject, from_email=from_email,
//                        to=recipients, connection=None,
//                        headers="")

//     msg.attach(attachment_filename, attachment, 'text/plain')

//     logging.info("Resending Message %s %s", recipients, msg)

//     try:
//         msg.send(fail_silently=False)
//     except Exception as err:
//         logging.warning(str(err))



// def report_hack(request):
//     start = arrow.Arrow(year=2014, month=11, day=1)
//     end = arrow.Arrow(year=2014, month=11, day=30)
//     for r in arrow.Arrow.span_range('day', start, end):
//         details = {}
//         details['report'] = {}
//         details['report']['name'] = 'FoodScience'
//         details['report']['subject'] = 'Multi-Building Test (Reading)'
//         details['report']['sensor_ids'] = [(422, 'RUSU'), (418, 'Bookshop'), (428, 'Chemistry'), (423, 'Archway_Lodge'),
//                                            (426, 'New_Business_School'), (431, 'Life_Sciences'), (424, 'ICMA'),
//                                            (429, 'Food_Bio'), (407, 'Whiteknights_House'), (413, 'Windsor_Hall')]
//         details['report']['recipients'] = ['glennpierce@gmail.com', 'j.m.burton@reading.ac.uk']
//         details['report']['headers'] = ''
//         details['report']['message'] = ''
//         run_report(details, r[0])


// if __name__ == "__main__":
//         details = {}
//         details['report'] = {}
//         details['report']['name'] = 'FoodScience'
//         details['report']['subject'] = 'Multi-Building Test (Reading)'
//         details['report']['sensor_ids'] = [(431, 'Life_Sciences'), (424, 'ICMA'),]
//         details['report']['recipients'] = ['glennpierce@gmail.com']
//         details['report']['headers'] = ''
//         details['report']['message'] = ''
//         run_report(details, dt=arrow.Arrow(year=2014, month=12, day=17))